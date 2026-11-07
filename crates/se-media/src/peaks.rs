//! Waveform peaks: minimum and maximum sample values per channel over fixed blocks of samples,
//! computed from interleaved `f32` PCM, and their binary file format (`SEPK`, documented in
//! `crates/se-media/CLAUDE.md`). Pure: no IO.

/// Samples per peak of the levels written for each asset, finest first; each level is 4× coarser.
pub const PEAK_LEVELS: [u32; 3] = [256, 1024, 4096];

/// File signature.
const MAGIC: &[u8; 4] = b"SEPK";
/// Format version written by this build.
const FORMAT_VERSION: u16 = 1;
/// Header size in bytes; the peaks start right after (aligned for `Int16Array`).
pub const HEADER_BYTES: usize = 32;
/// Scale from a sample in `[-1, 1]` to a stored `i16`.
const SCALE: f32 = 32767.0;

/// Peaks of one level: for each block, per channel, `(min, max)` scaled to `i16`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peaks {
    /// Channels per block.
    pub channels: u16,
    /// Samples per second of the source.
    pub sample_rate: u32,
    /// Samples (per channel) summarised by one peak.
    pub samples_per_peak: u32,
    /// `peak_count × channels` pairs, block-major: block 0 channel 0, block 0 channel 1…
    pub values: Vec<(i16, i16)>,
}

impl Peaks {
    /// Number of blocks.
    #[must_use]
    pub fn peak_count(&self) -> usize {
        self.values.len() / usize::from(self.channels.max(1))
    }

    /// The level `factor` times coarser (each block merges `factor` blocks; the last one may
    /// merge fewer).
    #[must_use]
    pub fn coarsen(&self, factor: u32) -> Self {
        let channels = usize::from(self.channels.max(1));
        let factor_blocks = usize::try_from(factor.max(1)).unwrap_or(1);
        let mut values = Vec::with_capacity(self.values.len() / factor_blocks + channels);
        for group in self.values.chunks(channels * factor_blocks) {
            for channel in 0..channels {
                let merged = group
                    .iter()
                    .skip(channel)
                    .step_by(channels)
                    .fold((i16::MAX, i16::MIN), |(low, high), (min, max)| {
                        (low.min(*min), high.max(*max))
                    });
                values.push(merged);
            }
        }
        Self {
            channels: self.channels,
            sample_rate: self.sample_rate,
            samples_per_peak: self.samples_per_peak.saturating_mul(factor),
            values,
        }
    }

    /// The `SEPK` file of these peaks.
    ///
    /// # Errors
    /// When there are more blocks than the format can count (`u32`).
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        let count = u32::try_from(self.peak_count())
            .map_err(|_| "too many peaks for the SEPK format".to_owned())?;
        let mut bytes = Vec::with_capacity(HEADER_BYTES + self.values.len() * 4);
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        bytes.extend_from_slice(&self.channels.to_le_bytes());
        bytes.extend_from_slice(&self.sample_rate.to_le_bytes());
        bytes.extend_from_slice(&self.samples_per_peak.to_le_bytes());
        bytes.extend_from_slice(&count.to_le_bytes());
        bytes.resize(HEADER_BYTES, 0);
        for (min, max) in &self.values {
            bytes.extend_from_slice(&min.to_le_bytes());
            bytes.extend_from_slice(&max.to_le_bytes());
        }
        Ok(bytes)
    }

    /// Reads a `SEPK` file.
    ///
    /// # Errors
    /// When the signature, version or length is wrong.
    pub fn decode(bytes: &[u8]) -> Result<Self, String> {
        let header = bytes
            .get(..HEADER_BYTES)
            .ok_or_else(|| "the file is shorter than the SEPK header".to_owned())?;
        if &header[..4] != MAGIC {
            return Err("not a SEPK file".to_owned());
        }
        let u16_at = |at: usize| u16::from_le_bytes([header[at], header[at + 1]]);
        let u32_at = |at: usize| {
            u32::from_le_bytes([header[at], header[at + 1], header[at + 2], header[at + 3]])
        };
        if u16_at(4) != FORMAT_VERSION {
            return Err(format!("unknown SEPK version {}", u16_at(4)));
        }
        let channels = u16_at(6);
        let count = usize::try_from(u32_at(16)).map_err(|error| error.to_string())?;
        let body = &bytes[HEADER_BYTES..];
        if body.len() != count * usize::from(channels) * 4 {
            return Err("the SEPK body does not match its peak count".to_owned());
        }
        let values = body
            .as_chunks::<4>()
            .0
            .iter()
            .map(|pair| {
                (
                    i16::from_le_bytes([pair[0], pair[1]]),
                    i16::from_le_bytes([pair[2], pair[3]]),
                )
            })
            .collect();
        Ok(Self {
            channels,
            sample_rate: u32_at(8),
            samples_per_peak: u32_at(12),
            values,
        })
    }
}

/// Builds the finest level from a stream of interleaved `f32` samples.
#[derive(Debug)]
pub struct PeakAccumulator {
    peaks: Peaks,
    /// Per channel `(min, max)` of the block being filled.
    current: Vec<(f32, f32)>,
    /// Frames (samples per channel) in the current block.
    frames_in_block: u32,
    /// Next channel of the interleaved stream.
    next_channel: usize,
    /// Frames pushed so far.
    frames: u64,
}

impl PeakAccumulator {
    /// An empty accumulator for `channels` channels (at least 1).
    #[must_use]
    pub fn new(channels: u16, sample_rate: u32, samples_per_peak: u32) -> Self {
        let channels = channels.max(1);
        Self {
            peaks: Peaks {
                channels,
                sample_rate,
                samples_per_peak: samples_per_peak.max(1),
                values: Vec::new(),
            },
            current: vec![(f32::MAX, f32::MIN); usize::from(channels)],
            frames_in_block: 0,
            next_channel: 0,
            frames: 0,
        }
    }

    /// Frames (samples per channel) pushed so far.
    #[must_use]
    pub fn frames(&self) -> u64 {
        self.frames
    }

    /// Adds interleaved samples (may stop in the middle of a frame).
    pub fn push(&mut self, samples: &[f32]) {
        let channels = self.current.len();
        for &sample in samples {
            let sample = if sample.is_finite() {
                sample.clamp(-1.0, 1.0)
            } else {
                0.0
            };
            let (low, high) = &mut self.current[self.next_channel];
            *low = low.min(sample);
            *high = high.max(sample);
            self.next_channel += 1;
            if self.next_channel == channels {
                self.next_channel = 0;
                self.frames += 1;
                self.frames_in_block += 1;
                if self.frames_in_block == self.peaks.samples_per_peak {
                    self.close_block();
                }
            }
        }
    }

    /// The peaks, the last partial block included.
    #[must_use]
    pub fn finish(mut self) -> Peaks {
        if self.frames_in_block > 0 {
            self.close_block();
        }
        self.peaks
    }

    fn close_block(&mut self) {
        for (low, high) in &mut self.current {
            self.peaks.values.push((to_i16(*low), to_i16(*high)));
            *low = f32::MAX;
            *high = f32::MIN;
        }
        self.frames_in_block = 0;
    }
}

/// A sample in `[-1, 1]` as a stored value.
fn to_i16(sample: f32) -> i16 {
    // Clamped to [-32767, 32767] before the cast, so it fits.
    #[allow(clippy::cast_possible_truncation)]
    let value = (sample * SCALE).round().clamp(-SCALE, SCALE) as i16;
    value
}

/// Every level of [`PEAK_LEVELS`] from the finest one.
#[must_use]
pub fn all_levels(finest: Peaks) -> Vec<Peaks> {
    let mut levels = vec![finest];
    for window in PEAK_LEVELS.windows(2) {
        let coarser = levels[levels.len() - 1].coarsen(window[1] / window[0]);
        levels.push(coarser);
    }
    levels
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A stereo signal: left is a full-scale square wave, right is silence.
    fn square_wave(frames: usize, period: usize) -> Vec<f32> {
        (0..frames)
            .flat_map(|frame| {
                let left = if (frame / (period / 2)).is_multiple_of(2) {
                    1.0
                } else {
                    -1.0
                };
                [left, 0.0]
            })
            .collect()
    }

    #[test]
    fn blocks_hold_the_extremes_of_each_channel() {
        let mut accumulator = PeakAccumulator::new(2, 48_000, 256);
        let samples = square_wave(1000, 100);
        // Pushed in uneven chunks, splitting frames.
        for chunk in samples.chunks(333) {
            accumulator.push(chunk);
        }
        assert_eq!(accumulator.frames(), 1000);
        let peaks = accumulator.finish();
        assert_eq!(
            peaks.peak_count(),
            4,
            "1000 frames = 3 full blocks + 1 partial"
        );
        assert_eq!(peaks.values[0], (-32767, 32767));
        assert_eq!(peaks.values[1], (0, 0));
    }

    #[test]
    fn coarser_levels_merge_four_blocks() {
        let mut accumulator = PeakAccumulator::new(1, 48_000, 256);
        let ramp: Vec<f32> = (0..256_u16 * 9)
            .map(|i| f32::from(i) / (256.0 * 9.0))
            .collect();
        accumulator.push(&ramp);
        let levels = all_levels(accumulator.finish());
        let sizes: Vec<(u32, usize)> = levels
            .iter()
            .map(|level| (level.samples_per_peak, level.peak_count()))
            .collect();
        assert_eq!(sizes, [(256, 9), (1024, 3), (4096, 1)]);
        let finest = &levels[0];
        assert_eq!(
            levels[1].values[0],
            (finest.values[0].0, finest.values[3].1)
        );
        assert_eq!(levels[2].values[0].1, finest.values[8].1);
    }

    #[test]
    fn files_round_trip_and_reject_garbage() {
        let peaks = Peaks {
            channels: 2,
            sample_rate: 44_100,
            samples_per_peak: 1024,
            values: vec![(-5, 9), (0, 0), (-32767, 32767), (1, 2)],
        };
        let bytes = peaks.encode().unwrap();
        assert_eq!(bytes.len(), HEADER_BYTES + 16);
        assert_eq!(&bytes[..4], b"SEPK");
        assert_eq!(Peaks::decode(&bytes).unwrap(), peaks);
        assert!(Peaks::decode(b"RIFF").is_err());
        assert!(Peaks::decode(&bytes[..bytes.len() - 1]).is_err());
    }

    #[test]
    fn out_of_range_samples_are_clamped() {
        let mut accumulator = PeakAccumulator::new(1, 8000, 2);
        accumulator.push(&[2.0, f32::NAN]);
        assert_eq!(accumulator.finish().values, [(0, 32767)]);
    }
}

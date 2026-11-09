//! Pictures sampled for face detection (prompt 26, ADR-0037): ffmpeg decodes the first picture
//! stream, keeps one frame per interval from the first frame on (`setpts=PTS-STARTPTS,fps=…`), turns
//! it upright (autorotation stays on), scales it down (`flags=area`) and writes raw BGR bytes on
//! stdout, read one frame at a time and sent to the detector in memory: nothing is written to
//! disk. Hardware decoding (`-hwaccel auto`) is tried first; a run that fails before its first
//! frame is redone in software.

use std::path::Path;

use se_app::ports::{FrameSampleRequest, MediaError, SampledFrame};
use se_domain::Flicks;
use tokio::{io::AsyncReadExt, sync::mpsc};

use crate::process::Sidecar;

/// The ffmpeg arguments of `request` (with hardware decoding or not).
#[must_use]
pub fn arguments(request: &FrameSampleRequest, hardware: bool) -> Vec<String> {
    let mut args: Vec<String> = ["-hide_banner", "-nostdin", "-v", "error"]
        .map(str::to_owned)
        .to_vec();
    if hardware {
        args.extend(["-hwaccel", "auto"].map(str::to_owned));
    }
    args.push("-i".to_owned());
    args.push(request.source.to_string_lossy().into_owned());
    let filters = format!(
        "setpts=PTS-STARTPTS,fps={},scale={}:{}:flags=area,format=bgr24",
        frame_rate(request.interval),
        request.width,
        request.height
    );
    args.extend(
        [
            "-map", "0:v:0", "-an", "-sn", "-dn", "-vf", &filters, "-f", "rawvideo", "pipe:1",
        ]
        .map(str::to_owned),
    );
    args
}

/// Samples per second for `interval`, as an exact ratio (`5/1` for 200 ms).
fn frame_rate(interval: Flicks) -> String {
    let numerator = Flicks::PER_SECOND;
    let denominator = interval.get().max(1);
    let divisor = gcd(numerator, denominator);
    format!("{}/{}", numerator / divisor, denominator / divisor)
}

const fn gcd(mut a: i64, mut b: i64) -> i64 {
    while b != 0 {
        let rest = a % b;
        a = b;
        b = rest;
    }
    if a == 0 { 1 } else { a }
}

/// Runs the sampling of `request` with `ffmpeg`, sending every frame to `frames`.
///
/// # Errors
/// [`MediaError::Failed`] when ffmpeg fails (in software too) or the size is unusable.
pub async fn sample(
    ffmpeg: &Path,
    request: &FrameSampleRequest,
    frames: &mpsc::Sender<SampledFrame>,
) -> Result<(), MediaError> {
    if request.hardware_decoding {
        match run(ffmpeg, request, true, frames).await {
            Ok(_) => return Ok(()),
            Err((0, error)) => {
                tracing::info!(%error, "hardware decoding failed before the first frame: software");
            }
            Err((_, error)) => return Err(error),
        }
    }
    run(ffmpeg, request, false, frames)
        .await
        .map(drop)
        .map_err(|(_, error)| error)
}

/// One ffmpeg run: the number of frames sent, or the number sent before the error.
async fn run(
    ffmpeg: &Path,
    request: &FrameSampleRequest,
    hardware: bool,
    frames: &mpsc::Sender<SampledFrame>,
) -> Result<u64, (u64, MediaError)> {
    let bytes = frame_bytes(request).ok_or_else(|| {
        let message = format!(
            "cannot sample pictures of {} × {} pixels",
            request.width, request.height
        );
        (0, MediaError::Failed(message))
    })?;
    let mut process =
        Sidecar::spawn(ffmpeg, arguments(request, hardware), None).map_err(|error| (0, error))?;
    let Some(mut stdout) = process.take_stdout() else {
        return Err((0, MediaError::Failed("ffmpeg gave no output".to_owned())));
    };
    let mut sent: u64 = 0;
    loop {
        let mut pixels = vec![0_u8; bytes];
        match stdout.read_exact(&mut pixels).await {
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(error) => {
                let message = format!("reading ffmpeg's pictures failed: {error}");
                return Err((sent, MediaError::Failed(message)));
            }
        }
        let frame = SampledFrame {
            time: sample_time(request.interval, sent),
            width: request.width,
            height: request.height,
            pixels,
        };
        if frames.send(frame).await.is_err() {
            // The consumer stopped: dropping the process kills ffmpeg.
            return Ok(sent);
        }
        sent += 1;
    }
    drop(stdout);
    process.finish().await.map_err(|error| (sent, error))?;
    Ok(sent)
}

/// Bytes of one frame, `None` for an empty or odd size.
fn frame_bytes(request: &FrameSampleRequest) -> Option<usize> {
    let valid = request.width > 0
        && request.height > 0
        && request.width.is_multiple_of(2)
        && request.height.is_multiple_of(2);
    if !valid {
        return None;
    }
    usize::try_from(u64::from(request.width) * u64::from(request.height) * 3).ok()
}

/// Source time of sample `index`.
fn sample_time(interval: Flicks, index: u64) -> Flicks {
    i64::try_from(index)
        .ok()
        .and_then(|index| interval.checked_mul(index))
        .unwrap_or(Flicks::MAX)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn request() -> FrameSampleRequest {
        FrameSampleRequest {
            source: PathBuf::from("in.mov"),
            interval: Flicks::from_millis(200),
            width: 640,
            height: 360,
            hardware_decoding: true,
        }
    }

    #[test]
    fn asks_for_upright_bgr_frames_at_the_sampling_rate() {
        let line = arguments(&request(), true).join(" ");
        assert!(line.contains("-hwaccel auto -i in.mov"), "{line}");
        assert!(line.contains("-map 0:v:0 -an -sn -dn"), "{line}");
        assert!(
            line.contains("-vf setpts=PTS-STARTPTS,fps=5/1,scale=640:360:flags=area,format=bgr24"),
            "{line}"
        );
        assert!(line.ends_with("-f rawvideo pipe:1"), "{line}");
        assert!(!line.contains("noautorotate"));
        assert!(!arguments(&request(), false).join(" ").contains("hwaccel"));
    }

    #[test]
    fn rates_are_exact_ratios() {
        assert_eq!(frame_rate(Flicks::from_millis(200)), "5/1");
        assert_eq!(frame_rate(Flicks::from_millis(300)), "10/3");
        assert_eq!(frame_rate(Flicks::one_second()), "1/1");
    }

    #[test]
    fn frames_have_a_size_and_a_time() {
        assert_eq!(frame_bytes(&request()), Some(640 * 360 * 3));
        let odd = FrameSampleRequest {
            width: 641,
            ..request()
        };
        assert_eq!(frame_bytes(&odd), None);
        assert_eq!(
            sample_time(Flicks::from_millis(200), 5),
            Flicks::one_second()
        );
    }
}

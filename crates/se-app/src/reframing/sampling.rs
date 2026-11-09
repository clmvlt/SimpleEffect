//! How a video is sampled for face detection (pure): the size of the pictures, the interval, and
//! which file is read. A ready proxy is read when there is one (same timing, much faster to
//! decode); hardware decoding is asked only for pictures larger than 1080p, where it pays: on
//! smaller ones its start-up and the copy of every decoded frame cost more than decoding on the
//! processor (measured on 1080p H.264 and 4K HEVC Main 10, ADR-0037).

use std::path::PathBuf;

use se_domain::{Flicks, VideoStreamInfo};

use crate::ports::FrameSampleRequest;

/// Time between two samples: 5 per second.
pub const SAMPLE_INTERVAL: Flicks = Flicks::from_millis(200);

/// Longest side of the sampled pictures: faces of a medium shot are 60–120 pixels high there,
/// well inside the detector's range (10–300).
pub const SAMPLE_LONGEST_SIDE: u32 = 640;

/// Pictures whose shorter side exceeds this are decoded by the hardware first.
const HARDWARE_DECODING_ABOVE: u32 = 1080;

/// The size of the pictures sampled from an upright picture of `width × height`: the longest
/// side at most [`SAMPLE_LONGEST_SIDE`] (never enlarged), both sides even and at least 2.
#[must_use]
pub fn sample_size(width: u32, height: u32) -> (u32, u32) {
    let longest = width.max(height).max(1);
    let scale = (f64::from(SAMPLE_LONGEST_SIDE) / f64::from(longest)).min(1.0);
    let even = |side: u32| {
        let scaled = (f64::from(side) * scale / 2.0).round() * 2.0;
        // Sides fit in u32 (at most the source's).
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let scaled = scaled as u32;
        scaled.max(2)
    };
    (even(width), even(height))
}

/// The sampling of a video whose picture is `stream`: its ready proxy when there is one, else
/// its original.
#[must_use]
pub fn sample_request(
    stream: &VideoStreamInfo,
    original: PathBuf,
    proxy: Option<PathBuf>,
) -> FrameSampleRequest {
    let (display_width, display_height) = stream.display_size();
    let (width, height) = sample_size(display_width, display_height);
    let heavy = display_width.min(display_height) > HARDWARE_DECODING_ABOVE;
    let hardware_decoding = heavy && proxy.is_none();
    FrameSampleRequest {
        source: proxy.unwrap_or(original),
        interval: SAMPLE_INTERVAL,
        width,
        height,
        hardware_decoding,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stream(width: u32, height: u32, rotation: u16) -> VideoStreamInfo {
        VideoStreamInfo {
            codec: "h264".to_owned(),
            profile: None,
            level: None,
            codec_tag: None,
            pixel_format: None,
            bit_depth: None,
            chroma_subsampling: None,
            width,
            height,
            frame_rate: None,
            rotation,
        }
    }

    #[test]
    fn pictures_are_scaled_to_640_on_their_longest_side() {
        assert_eq!(sample_size(1920, 1080), (640, 360));
        assert_eq!(sample_size(1080, 1920), (360, 640));
        assert_eq!(sample_size(3840, 1600), (640, 266));
        assert_eq!(sample_size(320, 240), (320, 240));
        assert_eq!(sample_size(0, 0), (2, 2));
    }

    #[test]
    fn proxies_are_preferred_and_large_originals_use_the_hardware() {
        let original = PathBuf::from("clip.mov");
        let proxy = PathBuf::from("proxy-540.mp4");
        let uhd = sample_request(
            &stream(3840, 2160, 0),
            original.clone(),
            Some(proxy.clone()),
        );
        assert_eq!(uhd.source, proxy);
        assert!(!uhd.hardware_decoding);
        let uhd = sample_request(&stream(3840, 2160, 0), original.clone(), None);
        assert_eq!(uhd.source, original);
        assert!(uhd.hardware_decoding);
        let hd = sample_request(&stream(1920, 1080, 0), original.clone(), None);
        assert!(!hd.hardware_decoding);
        let phone = sample_request(&stream(1920, 1080, 90), original, None);
        assert_eq!((phone.width, phone.height), (360, 640));
        assert_eq!(phone.interval, SAMPLE_INTERVAL);
    }
}

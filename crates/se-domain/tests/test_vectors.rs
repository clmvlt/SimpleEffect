//! Runs the shared vectors of `docs/test-vectors/` (flicks and timecodes) against se-domain. The
//! TypeScript time helpers (prompt 10) run the same files, which keeps both sides identical.

// Helpers outside #[test] functions must also fail loudly on malformed vectors.
#![allow(clippy::expect_used, clippy::panic)]

use se_domain::{
    Flicks, FrameRate, RoundingMode, TimeError, TimecodeErrorReason, format_timecode,
    parse_timecode,
};
use serde::Deserialize;

const FLICKS_VECTORS: &str = include_str!("../../../docs/test-vectors/flicks.json");
const TIMECODE_VECTORS: &str = include_str!("../../../docs/test-vectors/timecode.json");

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FlicksVectors {
    flicks_per_second: i64,
    max_safe: i64,
    frame_durations: Vec<FrameDuration>,
    reduced_frame_rates: Vec<ReducedFrameRate>,
    invalid_frame_rates: Vec<InvalidFrameRate>,
    frames_to_flicks: Vec<FramesToFlicks>,
    flicks_to_frames: Vec<FlicksToFrames>,
    seconds_to_flicks: Vec<SecondsToFlicks>,
    seconds_to_flicks_errors: Vec<SecondsToFlicksError>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FrameDuration {
    frame_rate: FrameRate,
    flicks: i64,
}

/// A frame rate as written in the vectors, before validation.
#[derive(Deserialize, Clone, Copy)]
struct RawRate {
    numerator: u32,
    denominator: u32,
}

#[derive(Deserialize)]
struct ReducedFrameRate {
    input: RawRate,
    reduced: FrameRate,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InvalidFrameRate {
    frame_rate: RawRate,
    error: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FramesToFlicks {
    frame_rate: FrameRate,
    frames: i64,
    flicks: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FlicksToFrames {
    frame_rate: FrameRate,
    flicks: i64,
    frames: FramesByRounding,
}

#[derive(Deserialize)]
struct FramesByRounding {
    floor: i64,
    ceil: i64,
    nearest: i64,
}

#[derive(Deserialize)]
struct SecondsToFlicks {
    seconds: f64,
    flicks: i64,
}

#[derive(Deserialize)]
struct SecondsToFlicksError {
    seconds: f64,
    error: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TimecodeVectors {
    parse: Vec<ParseCase>,
    parse_errors: Vec<ParseErrorCase>,
    format: Vec<FormatCase>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParseCase {
    input: String,
    frame_rate: Option<FrameRate>,
    flicks: i64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParseErrorCase {
    input: String,
    frame_rate: Option<FrameRate>,
    error: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FormatCase {
    flicks: i64,
    frame_rate: FrameRate,
    timecode: String,
}

fn flicks_vectors() -> FlicksVectors {
    serde_json::from_str(FLICKS_VECTORS).expect("docs/test-vectors/flicks.json is malformed")
}

fn timecode_vectors() -> TimecodeVectors {
    serde_json::from_str(TIMECODE_VECTORS).expect("docs/test-vectors/timecode.json is malformed")
}

fn flicks(value: i64) -> Flicks {
    Flicks::new(value).expect("vector time out of range")
}

/// The vector name of a time error: its variant in camelCase.
fn time_error_code(error: TimeError) -> &'static str {
    match error {
        TimeError::OutOfRange => "outOfRange",
        TimeError::NotFinite => "notFinite",
        TimeError::NegativeDuration { .. } => "negativeDuration",
        TimeError::ZeroFrameRate { .. } => "zeroFrameRate",
        TimeError::FrameRateOutOfRange { .. } => "frameRateOutOfRange",
        TimeError::InexactFrameRate { .. } => "inexactFrameRate",
    }
}

/// The vector name of a timecode error: its variant in camelCase.
fn timecode_error_code(reason: TimecodeErrorReason) -> &'static str {
    match reason {
        TimecodeErrorReason::Empty => "empty",
        TimecodeErrorReason::Unrecognised => "unrecognised",
        TimecodeErrorReason::MissingUnit => "missingUnit",
        TimecodeErrorReason::MinutesOutOfRange => "minutesOutOfRange",
        TimecodeErrorReason::SecondsOutOfRange => "secondsOutOfRange",
        TimecodeErrorReason::FramesOutOfRange { .. } => "framesOutOfRange",
        TimecodeErrorReason::FrameRateRequired => "frameRateRequired",
        TimecodeErrorReason::TooManyDecimals => "tooManyDecimals",
        TimecodeErrorReason::OutOfRange => "outOfRange",
    }
}

#[test]
fn constants_match() {
    let vectors = flicks_vectors();
    assert_eq!(vectors.flicks_per_second, Flicks::PER_SECOND);
    assert_eq!(vectors.max_safe, Flicks::MAX_SAFE);
}

#[test]
fn frame_rates() {
    let vectors = flicks_vectors();
    for case in vectors.frame_durations {
        assert_eq!(
            case.frame_rate.frame_duration(),
            flicks(case.flicks),
            "{}",
            case.frame_rate
        );
    }
    for case in vectors.reduced_frame_rates {
        assert_eq!(
            FrameRate::new(case.input.numerator, case.input.denominator),
            Ok(case.reduced)
        );
    }
    for case in vectors.invalid_frame_rates {
        let RawRate {
            numerator,
            denominator,
        } = case.frame_rate;
        let error = FrameRate::new(numerator, denominator)
            .expect_err(&format!("{numerator}/{denominator} should be rejected"));
        assert_eq!(
            time_error_code(error),
            case.error,
            "{numerator}/{denominator}"
        );
    }
}

#[test]
fn frame_conversions() {
    let vectors = flicks_vectors();
    for case in vectors.frames_to_flicks {
        assert_eq!(
            Flicks::from_frames(case.frames, case.frame_rate),
            Ok(flicks(case.flicks)),
            "{} frames at {}",
            case.frames,
            case.frame_rate
        );
    }
    for case in vectors.flicks_to_frames {
        let time = flicks(case.flicks);
        let expected = [
            (RoundingMode::Floor, case.frames.floor),
            (RoundingMode::Ceil, case.frames.ceil),
            (RoundingMode::Nearest, case.frames.nearest),
        ];
        for (rounding, frames) in expected {
            assert_eq!(
                time.to_frames(case.frame_rate, rounding),
                frames,
                "{} flicks at {} ({rounding:?})",
                case.flicks,
                case.frame_rate
            );
        }
    }
}

#[test]
fn seconds_conversions() {
    let vectors = flicks_vectors();
    for case in vectors.seconds_to_flicks {
        assert_eq!(
            Flicks::from_seconds_f64(case.seconds),
            Ok(flicks(case.flicks)),
            "{} s",
            case.seconds
        );
    }
    for case in vectors.seconds_to_flicks_errors {
        let error = Flicks::from_seconds_f64(case.seconds).expect_err("should be rejected");
        assert_eq!(time_error_code(error), case.error, "{} s", case.seconds);
    }
}

#[test]
fn timecode_parsing() {
    for case in timecode_vectors().parse {
        assert_eq!(
            parse_timecode(&case.input, case.frame_rate),
            Ok(flicks(case.flicks)),
            "{:?}",
            case.input
        );
    }
}

#[test]
fn timecode_parse_errors() {
    for case in timecode_vectors().parse_errors {
        let error = parse_timecode(&case.input, case.frame_rate)
            .expect_err(&format!("{:?} should be rejected", case.input));
        assert_eq!(
            timecode_error_code(error.reason),
            case.error,
            "{:?}",
            case.input
        );
    }
}

#[test]
fn timecode_formatting() {
    for case in timecode_vectors().format {
        assert_eq!(
            format_timecode(flicks(case.flicks), case.frame_rate),
            case.timecode,
            "{} flicks at {}",
            case.flicks,
            case.frame_rate
        );
    }
}

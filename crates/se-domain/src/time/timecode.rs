//! Human-readable times: parsing what users and AI agents type, formatting SMPTE timecodes.
//!
//! Accepted inputs (surrounding spaces ignored, an optional leading `-` negates):
//!
//! | Form | Example | Meaning |
//! |---|---|---|
//! | seconds | `51s`, `51.5s` | wall-clock seconds, up to 9 decimals |
//! | minutes:seconds | `00:51`, `1:02.500` | wall-clock time; the leading field is unbounded (`90:00`) |
//! | hours:minutes:seconds | `1:00:00`, `01:02:03.5` | wall-clock time |
//! | SMPTE frames | `00:00:51:12` | frame number at the frame rate (required) |
//! | SMPTE drop-frame | `00:00:51;12` | accepted, but read as non-drop for now |
//!
//! SMPTE timecodes count frames at the **nominal** rate ([`FrameRate::nominal_fps`]): at 29.97 fps
//! `00:00:01:00` is frame 30, which starts at 1.001 s. This is how non-drop-frame timecodes work in
//! every editor (Premiere included). Drop-frame counting (`;`) is not implemented yet: such inputs
//! are read as non-drop so they still parse (`docs/backlog.md`).
//!
//! A bare number (`51`) is refused as ambiguous. Decimal seconds are rounded to the nearest flick
//! (halves away from zero). Shared test vectors: `docs/test-vectors/timecode.json`, also used by
//! the TypeScript implementation for parity.

mod error;
mod parse;

pub use error::{TimecodeError, TimecodeErrorReason};

use super::{Flicks, FrameRate, RoundingMode};

/// Parses a time typed by a user or an agent (see the module documentation for the forms).
///
/// `rate` is needed only for SMPTE timecodes with a frames field.
///
/// # Errors
/// A [`TimecodeError`] quoting the input and explaining what to change.
pub fn parse_timecode(input: &str, rate: Option<FrameRate>) -> Result<Flicks, TimecodeError> {
    parse::parse(input.trim(), rate).map_err(|reason| TimecodeError {
        input: input.to_owned(),
        reason,
    })
}

/// Formats `time` as a non-drop SMPTE timecode `HH:MM:SS:FF` at `rate`.
///
/// The frame shown is the one displayed at that instant (rounded down). Negative times get a
/// leading `-`. Hours grow past two digits when needed; frames use at least two digits.
#[must_use]
pub fn format_timecode(time: Flicks, rate: FrameRate) -> String {
    let sign = if time.is_negative() { "-" } else { "" };
    let frames = time.abs().to_frames(rate, RoundingMode::Floor);
    let nominal = i64::from(rate.nominal_fps());
    let frame = frames % nominal;
    let total_seconds = frames / nominal;
    let seconds = total_seconds % 60;
    let minutes = total_seconds / 60 % 60;
    let hours = total_seconds / 3600;
    format!("{sign}{hours:02}:{minutes:02}:{seconds:02}:{frame:02}")
}

/// Formats `time` as wall-clock `HH:MM:SS.mmm`, rounded to the nearest millisecond (halves away
/// from zero). The format agents and error messages read; `parse_timecode` reads it back.
#[must_use]
pub fn format_clock(time: Flicks) -> String {
    const FLICKS_PER_MILLISECOND: i64 = Flicks::PER_SECOND / 1000;
    let sign = if time.is_negative() { "-" } else { "" };
    let magnitude = time.get().unsigned_abs();
    let unit = FLICKS_PER_MILLISECOND.unsigned_abs();
    let total_ms = (magnitude + unit / 2) / unit;
    let (hours, rest) = (total_ms / 3_600_000, total_ms % 3_600_000);
    let (minutes, rest) = (rest / 60_000, rest % 60_000);
    let (seconds, milliseconds) = (rest / 1000, rest % 1000);
    format!("{sign}{hours:02}:{minutes:02}:{seconds:02}.{milliseconds:03}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_times_read_back() {
        let time = at(51 * Flicks::PER_SECOND + Flicks::PER_SECOND / 4);
        assert_eq!(format_clock(time), "00:00:51.250");
        assert_eq!(parse_timecode(&format_clock(time), None), Ok(time));
        assert_eq!(format_clock(at(-Flicks::PER_SECOND)), "-00:00:01.000");
    }

    fn at(value: i64) -> Flicks {
        Flicks::new(value).unwrap()
    }

    #[test]
    fn parses_every_documented_form() {
        let second = Flicks::PER_SECOND;
        assert_eq!(parse_timecode("51s", None), Ok(at(51 * second)));
        assert_eq!(
            parse_timecode(" 51.5s ", None),
            Ok(at(51 * second + second / 2))
        );
        assert_eq!(parse_timecode("00:51", None), Ok(at(51 * second)));
        assert_eq!(
            parse_timecode("1:02.500", None),
            Ok(at(62 * second + second / 2))
        );
        assert_eq!(parse_timecode("1:00:00", None), Ok(at(3600 * second)));
        assert_eq!(parse_timecode("-2s", None), Ok(at(-2 * second)));
        assert_eq!(
            parse_timecode("00:00:51:12", Some(FrameRate::FPS_25)),
            Ok(at((51 * 25 + 12) * 28_224_000))
        );
    }

    #[test]
    fn drop_frame_notation_is_read_as_non_drop() {
        let rate = Some(FrameRate::FPS_29_97);
        assert_eq!(
            parse_timecode("00:00:51;12", rate),
            parse_timecode("00:00:51:12", rate)
        );
    }

    #[test]
    fn errors_quote_the_input_and_say_what_is_wrong() {
        let error = parse_timecode("1:75", None).unwrap_err();
        assert_eq!(error.reason, TimecodeErrorReason::SecondsOutOfRange);
        assert_eq!(error.to_string(), "'1:75': seconds must be < 60");
        let error = parse_timecode("00:00:01:30", Some(FrameRate::FPS_29_97)).unwrap_err();
        assert_eq!(
            error.to_string(),
            "'00:00:01:30': frames must be < 30 at 29.97 fps"
        );
    }

    #[test]
    fn formats_frames_at_the_nominal_rate() {
        assert_eq!(
            format_timecode(Flicks::ZERO, FrameRate::FPS_25),
            "00:00:00:00"
        );
        assert_eq!(
            format_timecode(at(1287 * 28_224_000), FrameRate::FPS_25),
            "00:00:51:12"
        );
        assert_eq!(
            format_timecode(at(-Flicks::PER_SECOND), FrameRate::FPS_25),
            "-00:00:01:00"
        );
        assert_eq!(
            format_timecode(at(3600 * Flicks::PER_SECOND), FrameRate::FPS_29_97),
            "00:59:56:12"
        );
    }

    #[test]
    fn formatting_then_parsing_lands_on_the_same_frame() {
        let rate = FrameRate::FPS_23_976;
        for frame in [0, 1, 23, 24, 1439, 86_399, 1_000_003] {
            let time = Flicks::from_frames(frame, rate).unwrap();
            assert_eq!(
                parse_timecode(&format_timecode(time, rate), Some(rate)),
                Ok(time)
            );
        }
    }
}

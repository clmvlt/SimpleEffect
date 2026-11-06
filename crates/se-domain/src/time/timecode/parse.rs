//! The timecode grammar (documented in the parent module). Works on magnitudes in `i128` flicks
//! so no intermediate value can overflow; the range is checked once at the end.

use super::TimecodeErrorReason::{
    self, Empty, FrameRateRequired, FramesOutOfRange, MinutesOutOfRange, MissingUnit, OutOfRange,
    SecondsOutOfRange, TooManyDecimals, Unrecognised,
};
use crate::time::{Flicks, FrameRate};

/// Most decimal places accepted for seconds (nanosecond precision, finer than one flick).
const MAX_DECIMALS: u32 = 9;

/// Parses an already trimmed input.
pub(super) fn parse(text: &str, rate: Option<FrameRate>) -> Result<Flicks, TimecodeErrorReason> {
    if text.is_empty() {
        return Err(Empty);
    }
    let (negative, body) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let magnitude = if let Some(seconds) = body.strip_suffix('s') {
        Decimal::parse(seconds)?.to_flicks()
    } else if body.contains([':', ';']) {
        parse_clock(body, rate)?
    } else if Decimal::parse(body).is_ok() {
        return Err(MissingUnit);
    } else {
        return Err(Unrecognised);
    };
    let signed = if negative { -magnitude } else { magnitude };
    i64::try_from(signed)
        .ok()
        .and_then(|value| Flicks::new(value).ok())
        .ok_or(OutOfRange)
}

/// Parses the colon forms: `MM:SS(.f)`, `HH:MM:SS(.f)` and `HH:MM:SS:FF` / `HH:MM:SS;FF`.
fn parse_clock(body: &str, rate: Option<FrameRate>) -> Result<i128, TimecodeErrorReason> {
    let fields: Vec<&str> = body.split([':', ';']).collect();
    if body.contains(';') && !is_drop_frame_notation(body, fields.len()) {
        return Err(Unrecognised);
    }
    match fields.as_slice() {
        [minutes, seconds] => {
            let seconds = Decimal::parse(seconds)?;
            check_below_60(seconds.integer, SecondsOutOfRange)?;
            Ok(
                whole_seconds_to_flicks(i128::from(parse_digits(minutes)?) * 60)
                    + seconds.to_flicks(),
            )
        }
        [hours, minutes, seconds] => {
            let minutes = parse_digits(minutes)?;
            check_below_60(minutes, MinutesOutOfRange)?;
            let seconds = Decimal::parse(seconds)?;
            check_below_60(seconds.integer, SecondsOutOfRange)?;
            let whole = i128::from(parse_digits(hours)?) * 3600 + i128::from(minutes) * 60;
            Ok(whole_seconds_to_flicks(whole) + seconds.to_flicks())
        }
        [hours, minutes, seconds, frames] => {
            parse_smpte([*hours, *minutes, *seconds, *frames], rate)
        }
        _ => Err(Unrecognised),
    }
}

/// `HH:MM:SS;FF`: exactly one `;`, placed before the frames field.
fn is_drop_frame_notation(body: &str, field_count: usize) -> bool {
    field_count == 4 && body.matches(';').count() == 1 && body.rfind(';') > body.rfind(':')
}

/// Parses `HH:MM:SS:FF`: frame number `((HH × 60 + MM) × 60 + SS) × nominal + FF` at `rate`.
fn parse_smpte(fields: [&str; 4], rate: Option<FrameRate>) -> Result<i128, TimecodeErrorReason> {
    let [hours, minutes, seconds, frames] = fields;
    let (hours, minutes, seconds) = (
        parse_digits(hours)?,
        parse_digits(minutes)?,
        parse_digits(seconds)?,
    );
    check_below_60(minutes, MinutesOutOfRange)?;
    check_below_60(seconds, SecondsOutOfRange)?;
    let frames = parse_digits(frames)?;
    let rate = rate.ok_or(FrameRateRequired)?;
    let nominal = rate.nominal_fps();
    if frames >= u64::from(nominal) {
        return Err(FramesOutOfRange {
            limit: nominal,
            rate,
        });
    }
    let whole_seconds = i128::from(hours) * 3600 + i128::from(minutes) * 60 + i128::from(seconds);
    let frame_number = whole_seconds * i128::from(nominal) + i128::from(frames);
    Ok(frame_number * i128::from(rate.frame_duration().get()))
}

fn check_below_60(value: u64, error: TimecodeErrorReason) -> Result<(), TimecodeErrorReason> {
    if value < 60 { Ok(()) } else { Err(error) }
}

fn whole_seconds_to_flicks(seconds: i128) -> i128 {
    seconds * i128::from(Flicks::PER_SECOND)
}

/// Whether `text` is a non-empty run of ASCII digits (no sign, no spaces).
fn is_digits(text: &str) -> bool {
    !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit())
}

/// Parses a non-empty run of ASCII digits.
fn parse_digits(text: &str) -> Result<u64, TimecodeErrorReason> {
    if !is_digits(text) {
        return Err(Unrecognised);
    }
    // Only digits are left, so the sole possible failure is an overflow.
    text.parse().map_err(|_| OutOfRange)
}

/// A non-negative decimal number of seconds: `51`, `51.5`, `0.000000001`.
struct Decimal {
    integer: u64,
    fraction: u64,
    decimals: u32,
}

impl Decimal {
    fn parse(text: &str) -> Result<Self, TimecodeErrorReason> {
        let (integer, fraction) = match text.split_once('.') {
            Some((integer, fraction)) => (integer, Some(fraction)),
            None => (text, None),
        };
        let integer = parse_digits(integer)?;
        let Some(fraction) = fraction else {
            return Ok(Self {
                integer,
                fraction: 0,
                decimals: 0,
            });
        };
        if !is_digits(fraction) {
            return Err(Unrecognised);
        }
        let decimals = u32::try_from(fraction.len())
            .ok()
            .filter(|&decimals| decimals <= MAX_DECIMALS)
            .ok_or(TooManyDecimals)?;
        Ok(Self {
            integer,
            fraction: parse_digits(fraction)?,
            decimals,
        })
    }

    /// The value in flicks, the fraction rounded to the nearest flick (halves up).
    fn to_flicks(&self) -> i128 {
        let scale = 10_i128.pow(self.decimals);
        let fraction = i128::from(self.fraction) * i128::from(Flicks::PER_SECOND);
        let rounded_fraction = (fraction * 2 + scale) / (scale * 2);
        whole_seconds_to_flicks(i128::from(self.integer)) + rounded_fraction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reason(text: &str, rate: Option<FrameRate>) -> TimecodeErrorReason {
        parse(text, rate).unwrap_err()
    }

    #[test]
    fn rejects_malformed_inputs() {
        assert_eq!(reason("", None), Empty);
        assert_eq!(reason("51", None), MissingUnit);
        assert_eq!(reason("5.5", None), MissingUnit);
        for text in [
            "abc",
            "s",
            "-",
            "--5s",
            "1.2.3s",
            ".5s",
            "5.s",
            "1:2:3:4:5",
            "1::2",
            "+5s",
        ] {
            assert_eq!(reason(text, None), Unrecognised, "{text}");
        }
    }

    #[test]
    fn drop_frame_separator_must_precede_the_frames() {
        let rate = Some(FrameRate::FPS_29_97);
        assert_eq!(reason("00;00:51:12", rate), Unrecognised);
        assert_eq!(reason("00:00;51;12", rate), Unrecognised);
        assert_eq!(reason("00:51;12", rate), Unrecognised);
    }

    #[test]
    fn checks_field_ranges() {
        assert_eq!(reason("1:60:00", None), MinutesOutOfRange);
        assert_eq!(reason("1:60", None), SecondsOutOfRange);
        assert_eq!(
            reason("00:00:60:00", Some(FrameRate::FPS_25)),
            SecondsOutOfRange
        );
        assert_eq!(reason("00:00:01:00", None), FrameRateRequired);
        assert_eq!(
            reason("00:00:01:25", Some(FrameRate::FPS_25)),
            FramesOutOfRange {
                limit: 25,
                rate: FrameRate::FPS_25
            }
        );
    }

    #[test]
    fn limits_decimals_and_range() {
        assert_eq!(reason("1.0000000001s", None), TooManyDecimals);
        assert_eq!(reason("20000000s", None), OutOfRange);
        assert_eq!(reason("99999999999999999999999s", None), OutOfRange);
    }

    #[test]
    fn rounds_fractions_to_the_nearest_flick() {
        // 1 ns is 0.7056 flick.
        assert_eq!(parse("0.000000001s", None).unwrap().get(), 1);
        assert_eq!(parse("-0.000000001s", None).unwrap().get(), -1);
    }

    #[test]
    fn leading_field_is_unbounded() {
        assert_eq!(
            parse("90:00", None).unwrap().get(),
            5400 * Flicks::PER_SECOND
        );
    }
}

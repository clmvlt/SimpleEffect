//! Times at the MCP boundary (golden rule 5): agents write seconds or timecodes and read
//! `{"timecode": "HH:MM:SS.mmm", "seconds": n}`; flicks never leave the core.

use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use se_domain::{Flicks, FrameRate, TimeRange, format_clock, parse_timecode};
use serde::{Deserialize, Serialize};

use crate::output::{ToolError, codes};

/// A time given by an agent: seconds (`51.5`) or a timecode string (`"51.5s"`, `"00:51"`,
/// `"00:00:51.500"`, `"00:00:51:12"` in frames of the sequence).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum TimeInput {
    /// Seconds.
    Seconds(f64),
    /// A timecode (`se_domain::parse_timecode` forms).
    Text(String),
}

impl JsonSchema for TimeInput {
    fn schema_name() -> Cow<'static, str> {
        "Time".into()
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "description": "Seconds (51.5) or a timecode string: \"51.5s\", \"00:51\", \
                            \"00:00:51.500\", or \"00:00:51:12\" (frames at the sequence's rate).",
            "oneOf": [{ "type": "number" }, { "type": "string" }]
        })
    }
}

impl TimeInput {
    /// The time in flicks; frame timecodes are read at `rate`.
    ///
    /// # Errors
    /// [`codes::INVALID_TIME`] quoting the input.
    pub fn to_flicks(&self, rate: FrameRate) -> Result<Flicks, ToolError> {
        match self {
            Self::Seconds(seconds) => Flicks::from_seconds_f64(*seconds).map_err(|_| {
                ToolError::new(
                    codes::INVALID_TIME,
                    format!("{seconds} is not a usable number of seconds"),
                )
            }),
            Self::Text(text) => parse_timecode(text, Some(rate)).map_err(|error| {
                ToolError::new(
                    codes::INVALID_TIME,
                    format!("{error}; write seconds (51.5) or a timecode (\"00:00:51.500\")"),
                )
            }),
        }
    }
}

/// A range given by an agent; the end is excluded.
#[derive(Debug, Clone, PartialEq, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RangeInput {
    /// First instant of the range.
    pub start: TimeInput,
    /// End of the range (excluded), after `start`.
    pub end: TimeInput,
}

impl RangeInput {
    /// The range in flicks.
    ///
    /// # Errors
    /// [`codes::INVALID_TIME`] for unreadable times, a negative start or an end not after the
    /// start.
    pub fn to_range(&self, rate: FrameRate) -> Result<TimeRange, ToolError> {
        let start = self.start.to_flicks(rate)?;
        let end = self.end.to_flicks(rate)?;
        if start.is_negative() {
            return Err(ToolError::new(
                codes::INVALID_TIME,
                format!("the range starts at {} s, before 0", start.to_seconds_f64()),
            ));
        }
        TimeRange::from_start_end(start, end)
            .ok()
            .filter(|range| !range.is_empty())
            .ok_or_else(|| {
                ToolError::new(
                    codes::INVALID_TIME,
                    format!(
                        "the range must end after it starts (start {}, end {})",
                        format_clock(start),
                        format_clock(end)
                    ),
                )
            })
    }
}

/// A time given back to an agent.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TimeOutput {
    /// `HH:MM:SS.mmm`.
    pub timecode: String,
    /// Seconds, rounded to the millisecond.
    pub seconds: f64,
}

impl From<Flicks> for TimeOutput {
    fn from(time: Flicks) -> Self {
        Self {
            timecode: format_clock(time),
            seconds: seconds(time),
        }
    }
}

/// A range given back to an agent.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RangeOutput {
    /// First instant.
    pub start: TimeOutput,
    /// End (excluded).
    pub end: TimeOutput,
}

impl From<TimeRange> for RangeOutput {
    fn from(range: TimeRange) -> Self {
        Self {
            start: range.start().into(),
            end: range.end().into(),
        }
    }
}

/// `time` in seconds, rounded to the millisecond (enough for agents, short in JSON).
#[must_use]
pub fn seconds(time: Flicks) -> f64 {
    (time.to_seconds_f64() * 1000.0).round() / 1000.0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(seconds: f64) -> Flicks {
        Flicks::from_seconds_f64(seconds).unwrap()
    }

    #[test]
    fn seconds_and_timecodes_read_alike() {
        let rate = FrameRate::FPS_25;
        let forms = [
            TimeInput::Seconds(51.0),
            TimeInput::Text("51s".to_owned()),
            TimeInput::Text("00:51".to_owned()),
            TimeInput::Text("00:00:51.000".to_owned()),
            TimeInput::Text("00:00:51:00".to_owned()),
        ];
        for form in forms {
            assert_eq!(form.to_flicks(rate).unwrap(), at(51.0), "{form:?}");
        }
        let error = TimeInput::Text("51".to_owned())
            .to_flicks(rate)
            .unwrap_err();
        assert_eq!(error.code, codes::INVALID_TIME);
        assert!(error.message.contains("'51'"), "{}", error.message);
        assert!(TimeInput::Seconds(f64::NAN).to_flicks(rate).is_err());
    }

    #[test]
    fn ranges_must_move_forward_from_zero() {
        let rate = FrameRate::FPS_25;
        let range = |start: f64, end: f64| RangeInput {
            start: TimeInput::Seconds(start),
            end: TimeInput::Seconds(end),
        };
        assert_eq!(
            range(1.0, 2.5).to_range(rate).unwrap(),
            TimeRange::from_start_end(at(1.0), at(2.5)).unwrap()
        );
        assert!(range(2.0, 2.0).to_range(rate).is_err());
        assert!(range(3.0, 2.0).to_range(rate).is_err());
        assert!(range(-1.0, 2.0).to_range(rate).is_err());
    }

    #[test]
    fn outputs_carry_a_timecode_and_rounded_seconds() {
        let output = TimeOutput::from(at(51.2504));
        assert_eq!(output.timecode, "00:00:51.250");
        assert!((output.seconds - 51.25).abs() < 1e-9);
        let json = serde_json::to_string(&RangeOutput::from(
            TimeRange::from_start_end(at(0.0), at(1.0)).unwrap(),
        ))
        .unwrap();
        assert_eq!(
            json,
            r#"{"start":{"timecode":"00:00:00.000","seconds":0.0},"end":{"timecode":"00:00:01.000","seconds":1.0}}"#
        );
    }
}

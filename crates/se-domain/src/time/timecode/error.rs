//! Errors of [`super::parse_timecode`].

use crate::time::FrameRate;

/// A time that could not be parsed, quoting the input: `'1:75': seconds must be < 60`.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("'{input}': {reason}")]
pub struct TimecodeError {
    /// The text as it was given (not trimmed).
    pub input: String,
    /// What is wrong with it.
    pub reason: TimecodeErrorReason,
}

/// Why a time could not be parsed. Messages tell the user or agent what to write instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TimecodeErrorReason {
    /// Nothing but spaces.
    #[error("the time is empty; write it like 51s, 51.5s, 00:51, 1:02.500 or 00:00:51:12")]
    Empty,
    /// None of the accepted forms.
    #[error("unrecognised time; write it like 51s, 51.5s, 00:51, 1:02.500 or 00:00:51:12")]
    Unrecognised,
    /// A number without unit, which could mean seconds or frames.
    #[error("a bare number is ambiguous; add the unit (51s) or write a timecode (00:51)")]
    MissingUnit,
    /// A minutes field after hours was 60 or more.
    #[error("minutes must be < 60")]
    MinutesOutOfRange,
    /// A seconds field after minutes was 60 or more.
    #[error("seconds must be < 60")]
    SecondsOutOfRange,
    /// A frames field reached the nominal frame rate.
    #[error("frames must be < {limit} at {rate} fps")]
    FramesOutOfRange {
        /// Frames per timecode second at this rate.
        limit: u32,
        /// The frame rate used to read the timecode.
        rate: FrameRate,
    },
    /// A SMPTE timecode with frames was given without a frame rate.
    #[error("a timecode with frames (HH:MM:SS:FF) needs the sequence frame rate")]
    FrameRateRequired,
    /// More than 9 decimal places of seconds.
    #[error("at most 9 decimal places are supported")]
    TooManyDecimals,
    /// The time is beyond ±`Flicks::MAX_SAFE`.
    #[error("the time is beyond the supported range (about ±147 days)")]
    OutOfRange,
}

//! Errors raised when a time value cannot be represented.

use super::Flicks;

/// Why a time, range or frame rate was rejected. Messages are read by users and AI agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TimeError {
    /// The value is beyond ±[`Flicks::MAX_SAFE`], the range JavaScript numbers hold exactly.
    #[error(
        "time is outside the supported range (±{} flicks, about ±147 days)",
        Flicks::MAX_SAFE
    )]
    OutOfRange,
    /// A float time was NaN or infinite.
    #[error("time must be a finite number of seconds")]
    NotFinite,
    /// A range was built with a negative duration.
    #[error("a time range cannot have a negative duration ({duration} flicks)")]
    NegativeDuration {
        /// The rejected duration, in flicks.
        duration: i64,
    },
    /// A frame rate had a zero numerator or denominator.
    #[error("frame rate {numerator}/{denominator} is invalid: both parts must be greater than 0")]
    ZeroFrameRate {
        /// Frames.
        numerator: u32,
        /// Seconds.
        denominator: u32,
    },
    /// A frame rate was slower than 1 fps or faster than 1000 fps.
    #[error(
        "frame rate {numerator}/{denominator} is not supported: it must be between 1 and 1000 \
         frames per second"
    )]
    FrameRateOutOfRange {
        /// Frames.
        numerator: u32,
        /// Seconds.
        denominator: u32,
    },
    /// One frame at this rate does not last a whole number of flicks.
    #[error(
        "frame rate {numerator}/{denominator} is not supported: one frame must last a whole \
         number of flicks; use a standard rate such as 24000/1001, 24, 25, 30000/1001, 30, 50, \
         60000/1001, 60 or 120"
    )]
    InexactFrameRate {
        /// Frames.
        numerator: u32,
        /// Seconds.
        denominator: u32,
    },
}

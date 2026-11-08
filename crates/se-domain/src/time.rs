//! Time in the document: integer flicks, rational frame rates, half-open ranges and timecodes
//! (ADR-0004, `docs/architecture.md` §5.1).
//!
//! Every instant and duration is a [`Flicks`]. Floats and frame counts exist only at boundaries
//! (UI, MCP tools, export writers), through the conversions of this module.

mod error;
mod flicks;
mod frame_rate;
mod rounding;
mod time_range;
mod timecode;

pub use error::TimeError;
pub use flicks::Flicks;
pub use frame_rate::FrameRate;
pub use rounding::RoundingMode;
pub use time_range::TimeRange;
pub use timecode::{
    TimecodeError, TimecodeErrorReason, format_clock, format_timecode, parse_timecode,
};

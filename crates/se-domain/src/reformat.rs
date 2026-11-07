//! Sequences in other formats (ADR-0026): changing a sequence's format in place and copying a
//! sequence (as is, or as a version in another format), with how items adapt to the new frame.
//!
//! - [`adaptation`]: [`FormatAdaptation`] (clip reframe policy, text widths) and the change it
//!   makes to one item ([`adapted_patch`]).
//! - [`copy`]: [`copy_sequence`], a copy with fresh ids handed out by a [`SequenceIds`].
//! - [`plan`]: [`format_change`] (the operations of an in-place change) and [`sequence_version`]
//!   (a copy in another format), both refusing a frame-rate change once a sequence holds items.
//!
//! Positions are normalised and sizes are design pixels (ADR-0015), so nothing else needs
//! converting: texts and graphics keep their relative places.

pub mod adaptation;
pub mod copy;
pub mod plan;

pub use adaptation::{ClipReframePolicy, FormatAdaptation, TEXT_WIDTH_SHARE, adapted_patch};
pub use copy::{SequenceIds, copy_sequence};
pub use plan::{format_change, sequence_version};

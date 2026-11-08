//! [`Violation`]: one broken invariant, with a stable code.

use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A broken document invariant, reported to users and AI agents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Violation {
    /// Stable, matchable code.
    pub code: ViolationCode,
    /// Where, in JSON field names with element ids:
    /// `sequences[<id>].tracks[<id>].items[<id>].kind.speed`; keyframes use their index.
    pub path: String,
    /// What is wrong and how to fix it.
    pub message: String,
}

impl fmt::Display for Violation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} at {}: {}",
            self.code, self.path, self.message
        )
    }
}

/// The invariant a [`Violation`] breaks. Serialized in `SCREAMING_SNAKE_CASE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ViolationCode {
    /// Two entities of the same kind share an id.
    DuplicateId,
    /// An id refers to an asset or template that does not exist.
    UnknownReference,
    /// An item refers to an asset of the wrong kind (a media clip of an image…).
    AssetKindMismatch,
    /// A sequence format has unusable dimensions or sample rate.
    InvalidFormat,
    /// A position (item start, source start, marker) is before zero.
    NegativeTime,
    /// A computed time (the end of a source range) leaves the supported range.
    TimeOutOfRange,
    /// An item has a zero duration.
    EmptyItem,
    /// Two items of the same track overlap in time.
    OverlappingItems,
    /// The items of a track are not sorted by start time.
    ItemsNotSorted,
    /// An item kind that the track kind cannot hold (a text on an audio track…).
    ItemNotAllowedOnTrack,
    /// A playback speed that is not a number greater than 0.
    InvalidSpeed,
    /// A media clip uses more of its asset than the asset contains.
    SourceRangeExceedsMedia,
    /// A keyframed property without keyframes.
    EmptyKeyframes,
    /// Keyframe times that are not strictly increasing.
    KeyframesNotSorted,
    /// A keyframe before the item's start or after its end.
    KeyframeOutsideItem,
    /// A value outside its allowed range (opacity, volume, pan, fades, Bézier control points).
    ValueOutOfRange,
    /// Items of one link group sit in different sequences.
    LinkGroupAcrossSequences,
    /// A transition that does not sit on the cut between two adjacent items of its track, or
    /// clashes with another transition.
    InvalidTransition,
    /// A transition needs more media beyond an item's edge than the item's asset has.
    InsufficientHandles,
    /// Caption words that are empty, overlap, are not in speaking order or leave their item.
    InvalidCaptionWords,
}

impl ViolationCode {
    /// Every code, in declaration order.
    pub const ALL: [Self; 20] = [
        Self::DuplicateId,
        Self::UnknownReference,
        Self::AssetKindMismatch,
        Self::InvalidFormat,
        Self::NegativeTime,
        Self::TimeOutOfRange,
        Self::EmptyItem,
        Self::OverlappingItems,
        Self::ItemsNotSorted,
        Self::ItemNotAllowedOnTrack,
        Self::InvalidSpeed,
        Self::SourceRangeExceedsMedia,
        Self::EmptyKeyframes,
        Self::KeyframesNotSorted,
        Self::KeyframeOutsideItem,
        Self::ValueOutOfRange,
        Self::LinkGroupAcrossSequences,
        Self::InvalidTransition,
        Self::InsufficientHandles,
        Self::InvalidCaptionWords,
    ];

    /// The serialized form (`OVERLAPPING_ITEMS`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DuplicateId => "DUPLICATE_ID",
            Self::UnknownReference => "UNKNOWN_REFERENCE",
            Self::AssetKindMismatch => "ASSET_KIND_MISMATCH",
            Self::InvalidFormat => "INVALID_FORMAT",
            Self::NegativeTime => "NEGATIVE_TIME",
            Self::TimeOutOfRange => "TIME_OUT_OF_RANGE",
            Self::EmptyItem => "EMPTY_ITEM",
            Self::OverlappingItems => "OVERLAPPING_ITEMS",
            Self::ItemsNotSorted => "ITEMS_NOT_SORTED",
            Self::ItemNotAllowedOnTrack => "ITEM_NOT_ALLOWED_ON_TRACK",
            Self::InvalidSpeed => "INVALID_SPEED",
            Self::SourceRangeExceedsMedia => "SOURCE_RANGE_EXCEEDS_MEDIA",
            Self::EmptyKeyframes => "EMPTY_KEYFRAMES",
            Self::KeyframesNotSorted => "KEYFRAMES_NOT_SORTED",
            Self::KeyframeOutsideItem => "KEYFRAME_OUTSIDE_ITEM",
            Self::ValueOutOfRange => "VALUE_OUT_OF_RANGE",
            Self::LinkGroupAcrossSequences => "LINK_GROUP_ACROSS_SEQUENCES",
            Self::InvalidTransition => "INVALID_TRANSITION",
            Self::InsufficientHandles => "INSUFFICIENT_HANDLES",
            Self::InvalidCaptionWords => "INVALID_CAPTION_WORDS",
        }
    }
}

impl fmt::Display for ViolationCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_matches_serialization_for_every_code() {
        for code in ViolationCode::ALL {
            assert_eq!(serde_json::to_string(&code).unwrap(), format!("\"{code}\""));
        }
    }
}

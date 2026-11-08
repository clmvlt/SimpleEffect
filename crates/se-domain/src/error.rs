//! [`DomainError`]: why an operation, a transaction, an undo or a redo was refused.
//!
//! Messages are read by users **and** relayed to AI agents: each one says what is wrong, quotes
//! the ids and times involved (as `HH:MM:SS.mmm`) and how to fix it. [`DomainError::code`] gives a
//! stable, matchable [`DomainErrorCode`].

mod code;

pub use code::DomainErrorCode;

use crate::{
    AnimatableProperty, AnimationPhase, AssetId, Flicks, FrameRate, ItemEdge, ItemId, MarkerId,
    SequenceId, TaskId, TrackId, TransitionId, Violation,
};

/// A refused change. Nothing was modified.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum DomainError {
    /// No item has this id.
    #[error("item {item} does not exist in this project; read the project again for current ids")]
    ItemNotFound {
        /// The unknown id.
        item: ItemId,
    },
    /// No track has this id.
    #[error("track {track} does not exist in this project; read the project again for current ids")]
    TrackNotFound {
        /// The unknown id.
        track: TrackId,
    },
    /// No sequence has this id.
    #[error(
        "sequence {sequence} does not exist in this project; read the project again for current \
         ids"
    )]
    SequenceNotFound {
        /// The unknown id.
        sequence: SequenceId,
    },
    /// No marker has this id.
    #[error(
        "marker {marker} does not exist in this project; read the project again for current ids"
    )]
    MarkerNotFound {
        /// The unknown id.
        marker: MarkerId,
    },
    /// No transition has this id.
    #[error(
        "transition {transition} does not exist in this project; it may have been removed with          its items, read the project again for current ids"
    )]
    TransitionNotFound {
        /// The unknown id.
        transition: TransitionId,
    },
    /// No asset has this id.
    #[error("asset {asset} does not exist in this project; import it or use an existing asset id")]
    AssetNotFound {
        /// The unknown id.
        asset: AssetId,
    },
    /// A new entity reuses an id of the project.
    #[error("{kind} id {id} is already used in this project; use a new id")]
    IdAlreadyUsed {
        /// Kind of entity (`item`, `track`…).
        kind: &'static str,
        /// The reused id.
        id: String,
    },
    /// A track given for a sequence-wide edit belongs to another sequence.
    #[error("track {track} is not in sequence {sequence}; list only tracks of the edited sequence")]
    TrackNotInSequence {
        /// The track.
        track: TrackId,
        /// The edited sequence.
        sequence: SequenceId,
    },
    /// The edit would make two items of a track overlap.
    #[error(
        "item {item} would overlap item {other} on track {track_name} between {} and {}; move or \
         trim one of them, or use another track",
        clock(.start),
        clock(.end)
    )]
    Overlap {
        /// The item placed.
        item: ItemId,
        /// The item already there.
        other: ItemId,
        /// Name of the track.
        track_name: String,
        /// Start of the overlap.
        start: Flicks,
        /// End of the overlap.
        end: Flicks,
    },
    /// The edit changes the items of a locked track or removes a locked track.
    #[error("track {track_name} ({track}) is locked; unlock it first")]
    TrackLocked {
        /// The track.
        track: TrackId,
        /// Its name.
        track_name: String,
    },
    /// The item changed since the caller read it (optimistic concurrency, ADR-0005).
    #[error(
        "item {item} is at revision {actual}, not {expected}: it changed since you read it; read \
         it again and retry"
    )]
    RevisionConflict {
        /// The item.
        item: ItemId,
        /// Revision the caller read.
        expected: u64,
        /// Current revision.
        actual: u64,
    },
    /// A trim would leave the item empty or reversed.
    #[error(
        "cannot trim the {edge} of item {item} to {}: it must stay {} its other edge at {}",
        clock(.to),
        edge.must_stay(),
        clock(.other_edge)
    )]
    TrimCollapsesItem {
        /// The item.
        item: ItemId,
        /// The trimmed edge.
        edge: ItemEdge,
        /// The requested position.
        to: Flicks,
        /// Position of the edge that does not move.
        other_edge: Flicks,
    },
    /// A trim would play a media clip beyond the start or end of its asset.
    #[error(
        "cannot trim the {edge} of item {item} to {}: the clip would play beyond the {edge} of its \
         asset; the furthest {edge} is {}",
        clock(.to),
        clock(.limit)
    )]
    TrimBeyondSource {
        /// The item.
        item: ItemId,
        /// The trimmed edge.
        edge: ItemEdge,
        /// The requested position.
        to: Flicks,
        /// The furthest allowed position, rounded to the millisecond on the safe side.
        limit: Flicks,
    },
    /// A split point is not strictly inside the item.
    #[error(
        "cannot split item {item} at {}: the split point must be strictly between its start {} \
         and its end {}",
        clock(.at),
        clock(.start),
        clock(.end)
    )]
    SplitOutsideItem {
        /// The item.
        item: ItemId,
        /// The requested split point.
        at: Flicks,
        /// Start of the item.
        start: Flicks,
        /// End of the item.
        end: Flicks,
    },
    /// An item patch of another kind than the item.
    #[error("item {item} is a {item_kind}; send a {item_kind} patch, not a {patch_kind} patch")]
    PatchKindMismatch {
        /// The item.
        item: ItemId,
        /// Kind of the item (`text`, `media clip`…).
        item_kind: &'static str,
        /// Kind of the patch.
        patch_kind: &'static str,
    },
    /// The item has no such animatable property.
    #[error("item {item} is a {item_kind}, which has no {property} property")]
    PropertyNotAvailable {
        /// The item.
        item: ItemId,
        /// Kind of the item.
        item_kind: &'static str,
        /// The requested property.
        property: AnimatableProperty,
    },
    /// An operation on a list of items got none.
    #[error("{operation} needs at least one item")]
    EmptyItemList {
        /// The operation (`linkItems`…).
        operation: &'static str,
    },
    /// An item appears twice in a list of items.
    #[error("item {item} is listed twice; list each item once")]
    DuplicateItem {
        /// The repeated item.
        item: ItemId,
    },
    /// A track order is not a permutation of the sequence's tracks.
    #[error(
        "the new track order of sequence {sequence} must list each of its {track_count} tracks \
         exactly once"
    )]
    InvalidTrackOrder {
        /// The sequence.
        sequence: SequenceId,
        /// Number of tracks it has.
        track_count: usize,
    },
    /// A time-based edit got an empty duration.
    #[error("{operation} needs a positive duration")]
    EmptyDuration {
        /// The operation (`rippleDeleteRange`, `insertGap`).
        operation: &'static str,
    },
    /// An item crossing an edit point must be split but no id was given for its new part.
    #[error(
        "item {item} crosses the edit point and must be split; give the id of its new part in \
         splitIds"
    )]
    MissingSplitId {
        /// The crossing item.
        item: ItemId,
    },
    /// The edit would move an item past the supported time range.
    #[error(
        "the edit would move item {item} beyond the supported time range (about 147 days); use \
         smaller times"
    )]
    TimeOutOfRange {
        /// The item.
        item: ItemId,
    },
    /// An asset still used by items cannot be removed.
    #[error("asset {asset} is used by item {item} (and maybe others); remove those items first")]
    AssetInUse {
        /// The asset.
        asset: AssetId,
        /// One item using it.
        item: ItemId,
    },
    /// Transcription settings with a malformed language or vocabulary.
    #[error("invalid transcription settings: {reason}")]
    InvalidTranscriptionSettings {
        /// What is wrong and how to fix it.
        reason: String,
    },
    /// A name that is empty or only spaces.
    #[error("a {entity} name cannot be empty")]
    EmptyName {
        /// Kind of entity (`project`, `sequence`, `track`).
        entity: &'static str,
    },
    /// An insertion index past the end of a list.
    #[error(
        "index {index} is past the end of the {entity} list ({len} entries); use 0 to {len}, or \
         null to append"
    )]
    IndexOutOfBounds {
        /// Kind of entity listed.
        entity: &'static str,
        /// The requested index.
        index: usize,
        /// Length of the list.
        len: usize,
    },
    /// The edit would break a document invariant (`validate_project`).
    #[error(
        "the change would leave the document invalid: {}",
        describe_violations(.violations)
    )]
    InvalidDocument {
        /// The broken invariants.
        violations: Vec<Violation>,
    },
    /// A transaction without operations.
    #[error("a transaction needs at least one operation")]
    EmptyTransaction,
    /// Undo with an empty undo stack.
    #[error("there is nothing to undo")]
    NothingToUndo,
    /// Redo with an empty redo stack.
    #[error("there is nothing to redo")]
    NothingToRedo,
    /// No animation preset has this id.
    #[error("there is no animation preset '{preset}'; use one of: {known}")]
    UnknownAnimationPreset {
        /// The unknown id.
        preset: String,
        /// Every preset id, comma-separated.
        known: String,
    },
    /// The preset cannot be applied in this phase (an emphasis as an entrance...).
    #[error("preset '{preset}' cannot be applied as {phase}; it supports: {supported}")]
    PresetPhaseNotSupported {
        /// The preset.
        preset: String,
        /// The phase requested.
        phase: AnimationPhase,
        /// Its phases, comma-separated.
        supported: String,
    },
    /// The item lacks a property the preset animates.
    #[error(
        "preset '{preset}' animates {property}, which item {item} (a {item_kind}) does not have; \
         choose a preset without it or apply it to another item"
    )]
    PresetNotApplicable {
        /// The item.
        item: ItemId,
        /// Kind of the item.
        item_kind: &'static str,
        /// The preset.
        preset: String,
        /// The property missing from the item.
        property: AnimatableProperty,
    },
    /// The embedded preset library could not be read (a bug, guarded by a test).
    #[error(
        "the animation preset library cannot be read ({reason}); this is a bug of SimpleEffect"
    )]
    PresetLibraryUnavailable {
        /// What is wrong with it.
        reason: String,
    },
    /// A frame-rate change of a sequence that already holds items.
    #[error(
        "sequence {sequence} already holds items placed at {current} fps, so its frame rate          cannot change to {requested} fps; keep {current} fps, or start from an empty sequence"
    )]
    FrameRateLocked {
        /// The sequence.
        sequence: SequenceId,
        /// Its frame rate.
        current: FrameRate,
        /// The frame rate asked for.
        requested: FrameRate,
    },
    /// The removal of the only sequence of a project.
    #[error(
        "sequence {sequence} is the only sequence of the project; create or duplicate another          sequence before removing it"
    )]
    LastSequence {
        /// The sequence.
        sequence: SequenceId,
    },
    /// A transaction recorded in the changeset of another task.
    #[error("only transactions of AI task {task} can be recorded in its changeset")]
    WrongChangeset {
        /// The changeset's task.
        task: TaskId,
    },
}

impl DomainError {
    /// The stable code of this error, for programs (`REVISION_CONFLICT`…).
    #[must_use]
    pub const fn code(&self) -> DomainErrorCode {
        match self {
            Self::ItemNotFound { .. } => DomainErrorCode::ItemNotFound,
            Self::TrackNotFound { .. } => DomainErrorCode::TrackNotFound,
            Self::SequenceNotFound { .. } => DomainErrorCode::SequenceNotFound,
            Self::MarkerNotFound { .. } => DomainErrorCode::MarkerNotFound,
            Self::AssetNotFound { .. } => DomainErrorCode::AssetNotFound,
            Self::TransitionNotFound { .. } => DomainErrorCode::TransitionNotFound,
            Self::IdAlreadyUsed { .. } => DomainErrorCode::IdAlreadyUsed,
            Self::TrackNotInSequence { .. } => DomainErrorCode::TrackNotInSequence,
            Self::Overlap { .. } => DomainErrorCode::OverlappingItems,
            Self::TrackLocked { .. } => DomainErrorCode::TrackLocked,
            Self::RevisionConflict { .. } => DomainErrorCode::RevisionConflict,
            Self::TrimCollapsesItem { .. } | Self::SplitOutsideItem { .. } => {
                DomainErrorCode::InvalidEditPoint
            }
            Self::TrimBeyondSource { .. } => DomainErrorCode::TrimBeyondSource,
            Self::PatchKindMismatch { .. }
            | Self::PropertyNotAvailable { .. }
            | Self::PresetNotApplicable { .. } => DomainErrorCode::WrongItemKind,
            Self::UnknownAnimationPreset { .. } => DomainErrorCode::UnknownAnimationPreset,
            Self::PresetPhaseNotSupported { .. } => DomainErrorCode::PresetPhaseNotSupported,
            Self::PresetLibraryUnavailable { .. } => DomainErrorCode::PresetLibraryUnavailable,
            Self::EmptyItemList { .. } | Self::DuplicateItem { .. } => {
                DomainErrorCode::InvalidItemList
            }
            Self::InvalidTrackOrder { .. } => DomainErrorCode::InvalidTrackOrder,
            Self::EmptyDuration { .. } => DomainErrorCode::EmptyDuration,
            Self::MissingSplitId { .. } => DomainErrorCode::MissingSplitId,
            Self::TimeOutOfRange { .. } => DomainErrorCode::TimeOutOfRange,
            Self::AssetInUse { .. } => DomainErrorCode::AssetInUse,
            Self::InvalidTranscriptionSettings { .. } => {
                DomainErrorCode::InvalidTranscriptionSettings
            }
            Self::EmptyName { .. } => DomainErrorCode::EmptyName,
            Self::IndexOutOfBounds { .. } => DomainErrorCode::IndexOutOfBounds,
            Self::InvalidDocument { .. } => DomainErrorCode::InvalidDocument,
            Self::EmptyTransaction => DomainErrorCode::EmptyTransaction,
            Self::NothingToUndo => DomainErrorCode::NothingToUndo,
            Self::NothingToRedo => DomainErrorCode::NothingToRedo,
            Self::WrongChangeset { .. } => DomainErrorCode::WrongChangeset,
            Self::FrameRateLocked { .. } => DomainErrorCode::FrameRateLocked,
            Self::LastSequence { .. } => DomainErrorCode::LastSequence,
        }
    }
}

/// A time for messages, as `HH:MM:SS.mmm` rounded to the nearest millisecond.
#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "thiserror passes fields to format helpers by reference"
)]
pub(crate) fn clock(time: &Flicks) -> String {
    crate::time::format_clock(*time)
}

fn describe_violations(violations: &[Violation]) -> String {
    violations
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::millis;

    #[test]
    fn clock_times_have_millisecond_precision() {
        assert_eq!(clock(&millis(51_000)), "00:00:51.000");
        assert_eq!(clock(&millis(3_723_456)), "01:02:03.456");
        assert_eq!(clock(&millis(-1500)), "-00:00:01.500");
        assert_eq!(clock(&Flicks::new(352_800).unwrap()), "00:00:00.001");
    }

    #[test]
    fn overlap_messages_quote_both_items_and_the_shared_range() {
        let error = DomainError::Overlap {
            item: ItemId::from_test_index(1),
            other: ItemId::from_test_index(2),
            track_name: "V1".to_owned(),
            start: millis(51_000),
            end: millis(52_500),
        };
        assert_eq!(
            error.to_string(),
            "item 00000000-0001-7000-8000-000000000000 would overlap item \
             00000000-0002-7000-8000-000000000000 on track V1 between 00:00:51.000 and \
             00:00:52.500; move or trim one of them, or use another track"
        );
        assert_eq!(error.code(), DomainErrorCode::OverlappingItems);
    }

    #[test]
    fn codes_display_like_their_serialization() {
        for code in [
            DomainErrorCode::RevisionConflict,
            DomainErrorCode::ItemNotFound,
            DomainErrorCode::InvalidDocument,
            DomainErrorCode::NothingToUndo,
        ] {
            assert_eq!(serde_json::to_string(&code).unwrap(), format!("\"{code}\""));
        }
        assert_eq!(
            DomainErrorCode::RevisionConflict.to_string(),
            "REVISION_CONFLICT"
        );
    }
}

//! [`DomainErrorCode`]: the stable, matchable code of a [`super::DomainError`].

use std::fmt;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Stable code of a [`super::DomainError`]. Serialized in `SCREAMING_SNAKE_CASE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DomainErrorCode {
    /// No item has the given id.
    ItemNotFound,
    /// No track has the given id.
    TrackNotFound,
    /// No sequence has the given id.
    SequenceNotFound,
    /// No marker has the given id.
    MarkerNotFound,
    /// No asset has the given id.
    AssetNotFound,
    /// No transition has the given id.
    TransitionNotFound,
    /// A new entity reuses an existing id.
    IdAlreadyUsed,
    /// A listed track belongs to another sequence.
    TrackNotInSequence,
    /// Two items of a track would overlap.
    OverlappingItems,
    /// A locked track would change.
    TrackLocked,
    /// The item changed since the caller read it.
    RevisionConflict,
    /// A trim or split position outside what the item allows.
    InvalidEditPoint,
    /// A trim beyond the media of a clip.
    TrimBeyondSource,
    /// A patch or property that does not fit the item's kind.
    WrongItemKind,
    /// An empty list of items, or an item listed twice.
    InvalidItemList,
    /// A track order that is not a permutation of the sequence's tracks.
    InvalidTrackOrder,
    /// A duration that must be positive is not.
    EmptyDuration,
    /// An item to split has no id for its new part.
    MissingSplitId,
    /// A time would leave the supported range.
    TimeOutOfRange,
    /// An asset still used by items.
    AssetInUse,
    /// An empty name.
    EmptyName,
    /// An insertion index past the end of a list.
    IndexOutOfBounds,
    /// The result would break a document invariant.
    InvalidDocument,
    /// A transaction without operations.
    EmptyTransaction,
    /// Nothing to undo.
    NothingToUndo,
    /// Nothing to redo.
    NothingToRedo,
    /// A transaction recorded in another task's changeset.
    WrongChangeset,
    /// No animation preset has the given id.
    UnknownAnimationPreset,
    /// An animation preset applied in a phase it does not have.
    PresetPhaseNotSupported,
    /// The embedded animation preset library is broken (a bug).
    PresetLibraryUnavailable,
    /// The frame rate of a sequence holding items cannot change.
    FrameRateLocked,
    /// The only sequence of a project cannot be removed.
    LastSequence,
    /// Transcription settings with a malformed language or vocabulary.
    InvalidTranscriptionSettings,
}

impl fmt::Display for DomainErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let debug = format!("{self:?}");
        let mut screaming = String::with_capacity(debug.len() + 4);
        for (index, character) in debug.char_indices() {
            if character.is_ascii_uppercase() && index > 0 {
                screaming.push('_');
            }
            screaming.push(character.to_ascii_uppercase());
        }
        formatter.write_str(&screaming)
    }
}

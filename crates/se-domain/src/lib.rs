//! # se-domain
//!
//! The pure document model of SimpleEffect: time in flicks, identifiers, projects,
//! sequences, tracks, items, keyframes and their validation.
//!
//! This crate is the innermost layer of the architecture (see `docs/architecture.md` §3 and §5):
//! every other crate may depend on it, and it depends on no workspace crate.
//!
//! ## Map
//! - [`time`]: [`Flicks`], [`FrameRate`], [`TimeRange`], timecode parsing and formatting.
//! - [`ids`]: one UUIDv7 newtype per entity kind ([`ItemId`], [`AssetId`]…).
//! - [`spatial`] and [`color`]: units of positions and sizes (ADR-0015), hex colours.
//! - [`document`]: [`Project`] and everything it contains.
//! - [`validation`]: [`validate_project`], the cross-entity invariants.
//! - [`operation`]: [`Operation`] and [`apply`], pure and invertible edits (`docs/architecture.md`
//!   §5.4, ADR-0005).
//! - [`transaction`]: [`Transaction`] and [`commit`], atomic groups of operations by one
//!   [`Actor`].
//! - [`history`]: [`History`], the user's undo/redo stacks.
//! - [`changeset`]: [`Changeset`] of an AI task and [`plan_revert`] with its [`Conflict`]s.
//! - [`error`]: [`DomainError`], why a change was refused.
//!
//! Every public type is re-exported at the crate root.
//!
//! ## Rules
//! - Pure: no IO, no async, no clock, no randomness, no process spawning.
//! - Allowed third-party dependencies: `serde`, `thiserror`, `uuid`, `ts-rs` (derive only).

pub mod changeset;
pub mod color;
pub mod document;
pub mod error;
pub mod history;
pub mod ids;
pub mod operation;
pub mod spatial;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod time;
pub mod transaction;
pub mod validation;

pub use changeset::{Changeset, ChangesetEntry, Conflict, RevertPlan, plan_revert};
pub use color::{Color, ColorParseError};
pub use document::{
    Animatable, Asset, AssetKind, AudioStreamInfo, BrandKit, CURRENT_SCHEMA_VERSION, CaptionItem,
    ContentFingerprint, DEFAULT_SAMPLE_RATE, Decodability, DerivedFile, DerivedMedia,
    DerivedStatus, Easing, EasingPreset, FingerprintError, Format, FormatPreset, ImageItem, Item,
    ItemKind, Keyframe, Marker, MarkerColor, MediaClip, MediaInfo, MotionTemplate, Origin, Project,
    ProjectSettings, Reframe, Sequence, ShapeGeometry, ShapeItem, StreamDecodability, TemplateItem,
    TemplateParamValue, TextItem, TextStyle, Track, TrackKind, Transform2D, VideoStreamInfo,
};
pub use error::{DomainError, DomainErrorCode};
pub use history::{History, HistoryStep};
pub use ids::{
    AssetId, ChangesetId, IdError, ItemId, JobId, LinkGroupId, MarkerId, ProjectId, SequenceId,
    TaskId, TemplateId, TrackId, TransactionId, Uuid,
};
pub use operation::{
    AddAsset, AddItem, AddMarker, AddSequence, AddTrack, AnimatableProperty, Applied, AssetPatch,
    Assign, CaptionPatch, ImagePatch, InsertGap, ItemEdge, ItemPatch, LinkItems, MarkerPatch,
    MediaClipPatch, MoveItem, Operation, PropertyAnimation, RemoveAsset, RemoveItem, RemoveMarker,
    RemoveSequence, RemoveTrack, RenameProject, ReorderTracks, RestoreItem, RippleDeleteRange,
    SequencePatch, SetKeyframes, ShapePatch, SplitItem, TemplatePatch, TextPatch, TouchedEntities,
    TrackPatch, TrimItem, UnlinkItems, UpdateAsset, UpdateItem, UpdateMarker, UpdateSequence,
    UpdateTrack, apply,
};
pub use spatial::{DESIGN_REFERENCE_SIDE, DesignPixels, Vec2};
pub use time::{
    Flicks, FrameRate, RoundingMode, TimeError, TimeRange, TimecodeError, TimecodeErrorReason,
    format_timecode, parse_timecode,
};
pub use transaction::{Actor, Committed, Transaction, commit};
pub use validation::{Violation, ViolationCode, validate_project};

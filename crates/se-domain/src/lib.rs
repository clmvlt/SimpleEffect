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
//!
//! Every public type is re-exported at the crate root. Operations, history and changesets arrive
//! with roadmap prompt 06.
//!
//! ## Rules
//! - Pure: no IO, no async, no clock, no randomness, no process spawning.
//! - Allowed third-party dependencies: `serde`, `thiserror`, `uuid`, `ts-rs` (derive only).

pub mod color;
pub mod document;
pub mod ids;
pub mod spatial;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod time;

pub use color::{Color, ColorParseError};
pub use document::{
    Animatable, Asset, AssetKind, AudioStreamInfo, BrandKit, CURRENT_SCHEMA_VERSION, CaptionItem,
    DEFAULT_SAMPLE_RATE, Decodability, DerivedFile, DerivedMedia, DerivedStatus, Easing,
    EasingPreset, Format, FormatPreset, ImageItem, Item, ItemKind, Keyframe, Marker, MarkerColor,
    MediaClip, MediaInfo, MotionTemplate, Origin, Project, ProjectSettings, Reframe, Sequence,
    ShapeGeometry, ShapeItem, TemplateItem, TemplateParamValue, TextItem, TextStyle, Track,
    TrackKind, Transform2D, VideoStreamInfo,
};
pub use ids::{
    AssetId, ChangesetId, IdError, ItemId, LinkGroupId, MarkerId, ProjectId, SequenceId, TaskId,
    TemplateId, TrackId, Uuid,
};
pub use spatial::{DESIGN_REFERENCE_SIDE, DesignPixels, Vec2};
pub use time::{
    Flicks, FrameRate, RoundingMode, TimeError, TimeRange, TimecodeError, TimecodeErrorReason,
    format_timecode, parse_timecode,
};

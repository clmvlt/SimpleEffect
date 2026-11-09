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
//! - [`analysis`]: results of media analyses ([`AudioAnalysis`], [`SceneDetection`]) and the
//!   summaries assets keep ([`AssetAnalysis`]).
//! - [`validation`]: [`validate_project`], the cross-entity invariants.
//! - [`operation`]: [`Operation`] and [`apply`], pure and invertible edits (`docs/architecture.md`
//!   §5.4, ADR-0005).
//! - [`reformat`]: sequences in other formats ([`format_change`], [`sequence_version`],
//!   [`copy_sequence`]; ADR-0026).
//! - [`transaction`]: [`Transaction`] and [`commit`], atomic groups of operations by one
//!   [`Actor`].
//! - [`history`]: [`History`], the user's undo/redo stacks.
//! - [`changeset`]: [`Changeset`] of an AI task and [`plan_revert`] with its [`Conflict`]s.
//! - [`caption_preset`] and [`captions`]: predefined caption styles, and the pure rules of
//!   captions (line breaking, segmentation of spoken words, text edits, merges and splits).
//! - [`templates`]: motion template definitions read from a module's exports
//!   ([`parse_definition`]) and item values carried over to a new version ([`migrate_params`]).
//! - [`transcript`]: words spoken in media ([`Transcript`]) and in a sequence
//!   ([`sequence_transcript`]).
//! - [`error`]: [`DomainError`], why a change was refused.
//!
//! Every public type is re-exported at the crate root.
//!
//! ## Rules
//! - Pure: no IO, no async, no clock, no randomness, no process spawning.
//! - Allowed third-party dependencies: `serde`, `thiserror`, `uuid`, `ts-rs` (derive only).

pub mod analysis;
pub mod animation_preset;
pub mod caption_preset;
pub mod captions;
pub mod changeset;
pub mod color;
pub mod document;
pub mod error;
pub mod history;
pub mod ids;
pub mod operation;
pub mod reformat;
pub mod spatial;
pub mod templates;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod time;
pub mod transaction;
pub mod transcript;
pub mod validation;

pub use analysis::{
    AnalysisKind, AnalysisResult, AssetAnalysis, AudioAnalysis, AudioSummary, LevelEnvelope,
    LoudnessCurve, SceneDetection, SceneSummary, SilenceSettings,
};
pub use animation_preset::{
    AnimationPhase, AnimationPreset, AnimationPresetLibrary, PresetPlacement, PresetRefusal,
    animation_presets, expand_preset, preset_samples,
};
pub use caption_preset::{
    CaptionStylePreset, DEFAULT_CAPTION_PRESET, caption_style_from_preset, caption_style_presets,
};
pub use changeset::{Changeset, ChangesetEntry, Conflict, RevertPlan, plan_revert};
pub use color::{Color, ColorParseError};
pub use document::{
    Animatable, AnimatableProperty, Asset, AssetDurations, AssetKind, AudioStreamInfo, BrandKit,
    CAPTION_LINE_CHARS, CAPTION_LINES, CURRENT_SCHEMA_VERSION, CaptionAnimation, CaptionBox,
    CaptionItem, CaptionOutline, CaptionShadow, CaptionStyle, CaptionWord, ContentFingerprint,
    DEFAULT_SAMPLE_RATE, DEFAULT_TEXT_MAX_LENGTH, Decodability, DerivedFile, DerivedMedia,
    DerivedStatus, Easing, EasingPreset, FadeShape, FingerprintError, Format, FormatPreset,
    Handles, ImageFit, ImageItem, Item, ItemKind, Keyframe, MAX_TEMPLATE_DESCRIPTION_CHARS,
    MAX_TEMPLATE_DURATION, MAX_TEMPLATE_NAME_CHARS, MAX_TEMPLATE_PARAMS, MAX_TEMPLATE_SOURCE_BYTES,
    Marker, MarkerColor, MediaClip, MediaInfo, MotionTemplate, Origin, Project, ProjectSettings,
    PropertyMut, PropertyRef, Reframe, Sequence, ShapeGeometry, ShapeItem, ShapeKind,
    StreamDecodability, Stroke, TemplateChoice, TemplateItem, TemplateParam, TemplateParamSpec,
    TemplateParamValue, TextAlign, TextBackground, TextCase, TextItem, TextShadow, TextStyle,
    Track, TrackKind, TranscriptionSettings, Transform2D, Transition, TransitionAlignment,
    TransitionKind, TransitionProblem, VideoStreamInfo, fitted_transition, is_param_name,
    item_handles, transition_problem, transition_room,
};
pub use error::{DomainError, DomainErrorCode};
pub use history::{History, HistoryStep};
pub use ids::{
    AssetId, CaptionStyleId, ChangesetId, IdError, ItemId, JobId, LinkGroupId, MarkerId, ProjectId,
    SequenceId, TaskId, TemplateId, TrackId, TransactionId, TransitionId, Uuid,
};
pub use operation::{
    AddAsset, AddCaptionStyle, AddItem, AddMarker, AddSequence, AddTemplate, AddTrack,
    AddTransition, Applied, ApplyAnimationPreset, AssetPatch, Assign, CaptionPatch,
    CaptionStylePatch, ImagePatch, InsertGap, ItemEdge, ItemPatch, LinkItems, MarkerPatch,
    MediaClipPatch, MoveItem, Operation, PropertyAnimation, RemoveAsset, RemoveCaptionStyle,
    RemoveItem, RemoveMarker, RemoveSequence, RemoveTemplate, RemoveTrack, RemoveTransition,
    RenameProject, ReorderTracks, ReplaceTemplate, RestoreItem, RippleDeleteRange,
    RippleDeleteRanges, SequencePatch, SetKeyframes, SetTranscriptionSettings, ShapePatch,
    SplitItem, TemplatePatch, TextPatch, TouchedEntities, TrackPatch, TransitionPatch,
    TransitionSnapshot, TrimItem, UnlinkItems, UpdateAsset, UpdateCaptionStyle, UpdateItem,
    UpdateMarker, UpdateSequence, UpdateTrack, UpdateTransition, apply, apply_all, remaining_parts,
};
pub use reformat::{
    ClipReframePolicy, FormatAdaptation, SequenceIds, TEXT_WIDTH_SHARE, adapted_patch,
    copy_sequence, format_change, sequence_version,
};
pub use spatial::{DESIGN_REFERENCE_SIDE, DesignPixels, Vec2};
pub use templates::{
    DefinitionProblem, ParamMigration, TemplateDefinition, migrate_params, parse_definition,
    parse_definition_json,
};
pub use time::{
    Flicks, FrameRate, RoundingMode, TimeError, TimeRange, TimecodeError, TimecodeErrorReason,
    format_clock, format_timecode, parse_timecode,
};
pub use transaction::{Actor, Committed, Transaction, commit};
pub use transcript::{
    ClipWindow, SequenceSegment, SequenceTranscript, SequenceWord, Transcript, TranscriptSegment,
    TranscriptSummary, TranscriptWord, join_words, sequence_transcript, words_in,
};
pub use validation::{Violation, ViolationCode, validate_project};

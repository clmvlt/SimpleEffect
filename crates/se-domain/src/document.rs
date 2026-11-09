//! The document model: a [`Project`] and everything it contains (`docs/architecture.md` §5.3).
//!
//! These are plain data types with public fields. Invariants that span several values (no overlap
//! on a track, references that exist, keyframes inside their item…) are checked by
//! [`crate::validate_project`]; invariants of a single value (a time in range, a valid frame rate,
//! a UUIDv7 id, a hex colour) are enforced by the value types themselves.
//!
//! Serialization (the `project.json` format and the protocol): fields in `camelCase`, enums with
//! data tagged by `kind`, unit enums as `camelCase` strings, times as flicks, `None` as `null`.

mod animatable;
mod animatable_property;
mod asset;
mod brand_kit;
mod caption;
mod caption_style;
mod content_fingerprint;
mod derived_media;
mod easing;
mod fade_shape;
mod format;
mod image;
mod item;
mod item_kind;
mod marker;
mod media_clip;
mod media_info;
mod motion_template;
mod project;
mod sequence;
mod shape;
mod stroke;
mod template_param;
mod text;
mod track;
mod transcription_settings;
mod transform;
mod transition;
mod transition_fit;

pub use animatable::{Animatable, Keyframe};
pub use animatable_property::{AnimatableProperty, PropertyMut, PropertyRef};
pub use asset::{Asset, AssetKind, Decodability, StreamDecodability};
pub use brand_kit::BrandKit;
pub use caption::{CaptionItem, CaptionWord};
pub use caption_style::{
    CAPTION_LINE_CHARS, CAPTION_LINES, CaptionAnimation, CaptionBox, CaptionOutline, CaptionShadow,
    CaptionStyle,
};
pub use content_fingerprint::{ContentFingerprint, FingerprintError};
pub use derived_media::{DerivedFile, DerivedMedia, DerivedStatus};
pub use easing::{Easing, EasingPreset};
pub use fade_shape::FadeShape;
pub use format::{DEFAULT_SAMPLE_RATE, Format, FormatPreset};
pub use image::{ImageFit, ImageItem};
pub use item::{Item, Origin};
pub use item_kind::ItemKind;
pub use marker::{Marker, MarkerColor};
pub use media_clip::{MediaClip, Reframe};
pub use media_info::{AudioStreamInfo, MediaInfo, VideoStreamInfo};
pub use motion_template::{
    MAX_TEMPLATE_DESCRIPTION_CHARS, MAX_TEMPLATE_DURATION, MAX_TEMPLATE_NAME_CHARS,
    MAX_TEMPLATE_SOURCE_BYTES, MotionTemplate, TemplateItem, TemplateParamValue,
};
pub use project::{CURRENT_SCHEMA_VERSION, Project, ProjectSettings};
pub use sequence::Sequence;
pub use shape::{ShapeGeometry, ShapeItem, ShapeKind};
pub use stroke::Stroke;
pub use template_param::{
    DEFAULT_TEXT_MAX_LENGTH, MAX_ENUM_OPTIONS, MAX_PARAM_NAME_CHARS, MAX_TEMPLATE_PARAMS,
    TemplateChoice, TemplateParam, TemplateParamSpec, is_param_name,
};
pub use text::{TextAlign, TextBackground, TextCase, TextItem, TextShadow, TextStyle};
pub use track::{Track, TrackKind};
pub use transcription_settings::TranscriptionSettings;
pub use transform::Transform2D;
pub use transition::{Transition, TransitionAlignment, TransitionKind};
pub use transition_fit::{
    AssetDurations, Handles, TransitionProblem, fitted_transition, item_handles,
    transition_problem, transition_room,
};

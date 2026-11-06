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
mod asset;
mod brand_kit;
mod caption;
mod derived_media;
mod easing;
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
mod text;
mod track;
mod transform;

pub use animatable::{Animatable, Keyframe};
pub use asset::{Asset, AssetKind, Decodability};
pub use brand_kit::BrandKit;
pub use caption::CaptionItem;
pub use derived_media::{DerivedFile, DerivedMedia, DerivedStatus};
pub use easing::{Easing, EasingPreset};
pub use format::{DEFAULT_SAMPLE_RATE, Format, FormatPreset};
pub use image::ImageItem;
pub use item::{Item, Origin};
pub use item_kind::ItemKind;
pub use marker::{Marker, MarkerColor};
pub use media_clip::{MediaClip, Reframe};
pub use media_info::{AudioStreamInfo, MediaInfo, VideoStreamInfo};
pub use motion_template::{MotionTemplate, TemplateItem, TemplateParamValue};
pub use project::{CURRENT_SCHEMA_VERSION, Project, ProjectSettings};
pub use sequence::Sequence;
pub use shape::{ShapeGeometry, ShapeItem};
pub use text::{TextItem, TextStyle};
pub use track::{Track, TrackKind};
pub use transform::Transform2D;

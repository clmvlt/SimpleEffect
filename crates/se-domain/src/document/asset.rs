//! [`Asset`]: a media file imported into a project.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{DerivedMedia, MediaInfo};
use crate::ids::AssetId;

/// A video, audio or image file used by the project. The file is referenced, never copied.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Asset {
    /// Stable id.
    pub id: AssetId,
    /// Absolute path of the user's file, in the native form of the OS that imported it.
    pub original_path: String,
    /// Path relative to the project folder, `/`-separated, used to relink the file when the
    /// project and its media move together. `None` when the file is on another drive.
    pub relative_hint: Option<String>,
    /// What the file contains; decides which items may use it.
    pub kind: AssetKind,
    /// Probed facts (duration, streams); `None` until the probe ran (prompt 08).
    pub media_info: Option<MediaInfo>,
    /// Mezzanine and proxy copies generated for this asset.
    pub derived: DerivedMedia,
    /// Whether the renderer can decode the original with WebCodecs.
    pub decodability: Decodability,
    /// The original was not found at its path or relative hint when the project was opened.
    pub missing: bool,
}

/// The kind of media an asset holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum AssetKind {
    /// Moving pictures, usually with sound.
    Video,
    /// Sound only.
    Audio,
    /// A still picture.
    Image,
}

/// Whether the original can be decoded in the renderer (checked with `isConfigSupported`).
///
/// TODO(prompt-08): refine per stream if a file can have a decodable video and an undecodable
/// audio stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum Decodability {
    /// Not checked yet.
    #[default]
    Unknown,
    /// WebCodecs decodes the original directly.
    Decodable,
    /// The original cannot be decoded: playback uses the mezzanine.
    NeedsMezzanine,
}

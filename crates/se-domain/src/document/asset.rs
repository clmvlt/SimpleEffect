//! [`Asset`]: a media file imported into a project.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{ContentFingerprint, DerivedMedia, MediaInfo};
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
    /// Probed facts (duration, streams); `None` until the probe ran.
    pub media_info: Option<MediaInfo>,
    /// Identity of the file's content when it was imported or relinked (deduplication, relinking,
    /// cache key of thumbnails and peaks); `None` only for documents older than schema version 2.
    pub fingerprint: Option<ContentFingerprint>,
    /// Mezzanine and proxy copies generated for this asset.
    pub derived: DerivedMedia,
    /// Whether the renderer can decode each stream of the original with WebCodecs.
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

/// Whether the renderer of this machine can decode each stream of the original, as it reported
/// after checking with WebCodecs (`isConfigSupported`, `docs/architecture.md` §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Decodability {
    /// The picture stream (or the still picture).
    pub video: StreamDecodability,
    /// The first sound stream.
    pub audio: StreamDecodability,
}

impl Decodability {
    /// The state of a newly probed asset: `unknown` for the streams `info` lists, `noStream` for
    /// the others.
    #[must_use]
    pub fn unchecked(info: &MediaInfo) -> Self {
        let state = |present: bool| {
            if present {
                StreamDecodability::Unknown
            } else {
                StreamDecodability::NoStream
            }
        };
        Self {
            video: state(info.video_stream.is_some()),
            audio: state(!info.audio_streams.is_empty()),
        }
    }

    /// Whether a stream still waits for the renderer's check.
    #[must_use]
    pub fn is_pending(self) -> bool {
        self.video == StreamDecodability::Unknown || self.audio == StreamDecodability::Unknown
    }
}

/// Whether one stream of an original can be decoded by the renderer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum StreamDecodability {
    /// Not checked yet.
    #[default]
    Unknown,
    /// WebCodecs decodes it directly.
    Decodable,
    /// WebCodecs cannot decode it: playback needs a mezzanine (prompt 09).
    Undecodable,
    /// The file has no such stream.
    NoStream,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_assets_wait_for_the_check_of_the_streams_they_have() {
        let info = MediaInfo {
            duration: None,
            container: "wav".to_owned(),
            video_stream: None,
            audio_streams: vec![crate::AudioStreamInfo {
                codec: "pcm_s16le".to_owned(),
                profile: None,
                channels: 2,
                sample_rate: 48_000,
            }],
        };
        let decodability = Decodability::unchecked(&info);
        assert_eq!(decodability.video, StreamDecodability::NoStream);
        assert_eq!(decodability.audio, StreamDecodability::Unknown);
        assert!(decodability.is_pending());
        let checked = Decodability {
            audio: StreamDecodability::Decodable,
            ..decodability
        };
        assert!(!checked.is_pending());
    }

    #[test]
    fn serializes_streams_by_name() {
        let json = serde_json::to_string(&Decodability {
            video: StreamDecodability::Undecodable,
            audio: StreamDecodability::NoStream,
        })
        .unwrap();
        assert_eq!(json, r#"{"video":"undecodable","audio":"noStream"}"#);
    }
}

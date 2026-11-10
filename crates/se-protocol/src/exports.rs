//! Video exports (prompt 27, ADR-0038).
//!
//! The core orchestrates an export and writes its file; the renderer draws it with `@se/engine`
//! (golden rule 7: the export is the preview). `startExport` snapshots the sequence and queues an
//! export job; when the job runs, the core publishes [`crate::CoreEvent::ExportRenderRequested`]
//! with that snapshot, the renderer renders and uploads (`PUT /media/exports/{id}/…`), reports its
//! progress and tells the core it finished (or why it failed); the core then completes the file
//! (rename, or ffmpeg for ProRes/DNxHR and AAC) and checks it with ffprobe.

use se_domain::{ExportId, Flicks, JobId, ProjectId, SequenceId, TimeRange};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::ProjectSnapshot;

/// The container of an exported file (its extension).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ExportContainer {
    /// `.mp4`: H.264, HEVC or AV1 with AAC or Opus.
    Mp4,
    /// `.mov`: H.264, HEVC, ProRes or DNxHR with AAC or PCM.
    Mov,
    /// `.webm`: VP9 or AV1 with Opus.
    Webm,
    /// `.m4a`: AAC sound only.
    M4a,
    /// `.wav`: PCM sound only.
    Wav,
}

/// The picture codec of an export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum VideoExportCodec {
    /// H.264 (WebCodecs).
    H264,
    /// HEVC (WebCodecs, hardware encoders only on most machines).
    Hevc,
    /// AV1 (WebCodecs).
    Av1,
    /// VP9 (WebCodecs).
    Vp9,
    /// Apple ProRes 422 HQ, 10 bits (ffmpeg `prores_ks`): a master.
    Prores422Hq,
    /// Apple ProRes 4444, 10 bits (ffmpeg `prores_ks`).
    Prores4444,
    /// Avid DNxHR HQ, 8 bits (ffmpeg `dnxhd`).
    DnxhrHq,
}

/// A quality level, when the bitrate is left to the encoder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum ExportQuality {
    /// Small files.
    Low,
    /// Balanced.
    Medium,
    /// The default of most presets.
    High,
    /// Close to the source.
    VeryHigh,
}

/// How the picture's bitrate is chosen (ignored by ProRes and DNxHR, whose profiles fix it).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum VideoRate {
    /// A target bitrate.
    Bitrate {
        /// Bits per second.
        bits_per_second: u32,
    },
    /// A quality level (the encoder picks the bitrate).
    Quality {
        /// The level.
        level: ExportQuality,
    },
}

/// The picture of an export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct VideoExportSettings {
    /// Its codec.
    pub codec: VideoExportCodec,
    /// Width in pixels (even); the sequence frame is scaled to it.
    pub width: u32,
    /// Height in pixels (even).
    pub height: u32,
    /// Its bitrate.
    pub rate: VideoRate,
}

/// The sound codec of an export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum AudioExportCodec {
    /// AAC-LC.
    Aac,
    /// Opus.
    Opus,
    /// 16-bit PCM (WAV files, ProRes and DNxHR masters).
    Pcm,
}

/// Who encodes the sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum AudioEncoderChoice {
    /// The renderer (WebCodecs `AudioEncoder`, or Mediabunny for PCM).
    WebCodecs,
    /// ffmpeg in the core, from a WAV the renderer writes: AAC where WebCodecs cannot encode it.
    Ffmpeg,
}

/// The sound of an export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AudioExportSettings {
    /// Its codec.
    pub codec: AudioExportCodec,
    /// Bits per second (ignored for PCM).
    pub bits_per_second: u32,
    /// Who encodes it.
    pub encoder: AudioEncoderChoice,
}

/// What an export contains and how it is encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ExportSettings {
    /// The file's container.
    pub container: ExportContainer,
    /// The picture, or `null` for a sound-only file.
    pub video: Option<VideoExportSettings>,
    /// The sound, or `null` for a silent file.
    pub audio: Option<AudioExportSettings>,
}

/// An export to start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct StartExportRequest {
    /// The open project.
    pub project_id: ProjectId,
    /// The sequence to export.
    pub sequence_id: SequenceId,
    /// What to write.
    pub settings: ExportSettings,
    /// The part of the sequence (its in/out span), or the whole sequence when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub range: Option<TimeRange>,
    /// Absolute path of the file to write (replaced when it exists); its extension must match the
    /// container.
    pub path: String,
    /// Name of the preset, shown in the exports list.
    pub preset_name: String,
}

/// How the file of an export is put together (decided by the core from the settings).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ExportPipeline {
    /// The renderer muxes the whole file (`PUT …/data`).
    Muxed,
    /// The renderer uploads RGBA frames (`PUT …/frames`) that ffmpeg encodes (ProRes, DNxHR),
    /// and first the sound as a WAV (`PUT …/audio`) when there is sound.
    FfmpegFrames {
        /// Whether a WAV comes first.
        with_audio: bool,
    },
    /// The renderer muxes the picture (`PUT …/data`, when there is one) and writes the sound as a
    /// WAV (`PUT …/audio`); ffmpeg encodes the sound as AAC and joins both.
    AacRemux {
        /// Whether a picture file is written too.
        with_video: bool,
    },
}

/// What the renderer must render for an export (event `exportRenderRequested`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ExportRenderRequest {
    /// The export (upload routes, progress and outcome reports).
    pub export_id: ExportId,
    /// Its project.
    pub project_id: ProjectId,
    /// The exported sequence.
    pub sequence_id: SequenceId,
    /// The sequence as it was when the export started: that sequence, its tracks and items, the
    /// media they use and the project's header (templates, caption styles). Later edits never
    /// reach the export.
    pub snapshot: ProjectSnapshot,
    /// What to write.
    pub settings: ExportSettings,
    /// Sequence time of the first frame.
    pub start: Flicks,
    /// Sequence time where the export ends (exclusive).
    pub end: Flicks,
    /// How the file is put together.
    pub pipeline: ExportPipeline,
}

/// What ffprobe found in a finished export.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ExportCheck {
    /// Duration of the file, in seconds.
    pub duration_seconds: f64,
    /// Codec of its picture (`h264`, `prores`…), if it has one.
    pub video_codec: Option<String>,
    /// Size of its picture, if it has one.
    pub width: Option<u32>,
    /// Height of its picture.
    pub height: Option<u32>,
    /// Codec of its sound (`aac`, `opus`, `pcm_s16le`…), if it has one.
    pub audio_codec: Option<String>,
    /// What looks wrong (a duration or stream that differs from what was asked), if anything.
    pub warning: Option<String>,
}

/// Where an export stands.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ExportState {
    /// Waiting for the export before it (one export renders at a time).
    Queued,
    /// The renderer is drawing and encoding.
    Rendering {
        /// Fraction done in `[0, 1]`.
        progress: f64,
        /// Seconds left, once they can be estimated.
        eta_seconds: Option<f64>,
    },
    /// The core completes the file (ffmpeg, rename, check).
    Finalizing,
    /// The file is written.
    Completed {
        /// Its size in bytes.
        size_bytes: u64,
        /// Seconds from the start of the render to the end.
        elapsed_seconds: f64,
        /// What ffprobe found, `null` when it could not read the file.
        check: Option<ExportCheck>,
    },
    /// The export failed; its partial files were removed.
    Failed {
        /// Why (actionable).
        message: String,
    },
    /// Cancelled by the user or because its project closed; its partial files were removed.
    Cancelled,
}

/// An export as clients see it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ExportInfo {
    /// Stable id.
    pub id: ExportId,
    /// Its job in the core's job queue (`null` only while it is being queued).
    pub job_id: Option<JobId>,
    /// Its project.
    pub project_id: ProjectId,
    /// The exported sequence.
    pub sequence_id: SequenceId,
    /// The sequence's name when the export started.
    pub sequence_name: String,
    /// The preset it was started with.
    pub preset_name: String,
    /// Absolute path of the file.
    pub path: String,
    /// What it writes.
    pub settings: ExportSettings,
    /// Duration of the exported span.
    pub duration: Flicks,
    /// When it was started (Unix milliseconds).
    pub created_at: u64,
    /// Whether a picture of it is served at `/media/exports/{id}/thumbnail`.
    pub has_thumbnail: bool,
    /// Where it stands.
    pub state: ExportState,
}

/// Picture settings of a saved preset: the output keeps the sequence's aspect, the preset fixes
/// its shorter side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ExportPresetVideo {
    /// Its codec.
    pub codec: VideoExportCodec,
    /// The shorter side in pixels (1080 for « 1080p »).
    pub short_side: u32,
    /// Its bitrate.
    pub rate: VideoRate,
}

/// Settings the user saved under a name (« Réglages perso »).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ExportPreset {
    /// Stable id (chosen by the renderer).
    pub id: String,
    /// Name shown in the preset list.
    pub name: String,
    /// The file's container.
    pub container: ExportContainer,
    /// The picture, or `null` for sound only.
    pub video: Option<ExportPresetVideo>,
    /// The sound, or `null` for silence.
    pub audio: Option<AudioExportSettings>,
}

impl ExportPreset {
    /// `presets` as JSON text (how the app's library stores them); `None` if serde fails.
    #[must_use]
    pub fn list_to_json(presets: &[Self]) -> Option<String> {
        serde_json::to_string(presets).ok()
    }

    /// The presets of JSON text written by [`Self::list_to_json`]; `None` when unreadable.
    #[must_use]
    pub fn list_from_json(text: &str) -> Option<Vec<Self>> {
        serde_json::from_str(text).ok()
    }
}

/// What the export dialog remembers between exports (the app's library).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ExportPreferences {
    /// The folder exports are written to (the last one used; the user's videos folder at first).
    pub directory: String,
    /// The file name template (`{project}`, `{sequence}`, `{preset}`, `{date}`, `{time}`,
    /// `{width}`, `{height}`).
    pub file_name_template: String,
    /// The user's saved presets.
    pub presets: Vec<ExportPreset>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_are_camel_case_and_tagged() {
        let settings = ExportSettings {
            container: ExportContainer::Mov,
            video: Some(VideoExportSettings {
                codec: VideoExportCodec::Prores422Hq,
                width: 1920,
                height: 1080,
                rate: VideoRate::Quality {
                    level: ExportQuality::VeryHigh,
                },
            }),
            audio: Some(AudioExportSettings {
                codec: AudioExportCodec::Pcm,
                bits_per_second: 0,
                encoder: AudioEncoderChoice::WebCodecs,
            }),
        };
        let json = serde_json::to_value(settings).unwrap();
        assert_eq!(json["container"], "mov");
        assert_eq!(json["video"]["codec"], "prores422Hq");
        assert_eq!(json["video"]["rate"]["kind"], "quality");
        assert_eq!(json["video"]["rate"]["level"], "veryHigh");
        assert_eq!(json["audio"]["encoder"], "webCodecs");
        assert_eq!(json["audio"]["bitsPerSecond"], 0);
    }

    #[test]
    fn states_and_pipelines_are_tagged_by_kind() {
        let state = ExportState::Rendering {
            progress: 0.5,
            eta_seconds: None,
        };
        assert_eq!(
            serde_json::to_string(&state).unwrap(),
            r#"{"kind":"rendering","progress":0.5,"etaSeconds":null}"#
        );
        assert_eq!(
            serde_json::to_string(&ExportPipeline::AacRemux { with_video: true }).unwrap(),
            r#"{"kind":"aacRemux","withVideo":true}"#
        );
    }
}

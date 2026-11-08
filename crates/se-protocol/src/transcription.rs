//! Transcription DTOs (prompt 21, ADR-0031): this machine's transcription status (whisper.cpp,
//! GPU, models and downloads), the app's transcription preferences and transcript search hits.
//! Transcripts themselves are the domain's [`se_domain::Transcript`].

use se_domain::{AssetId, Flicks};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Where whisper.cpp computes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum TranscriptionBackend {
    /// An NVIDIA GPU through CUDA.
    Cuda,
    /// An Apple GPU through Metal.
    Metal,
    /// A GPU through Vulkan (user-provided builds).
    Vulkan,
    /// The processor.
    Cpu,
}

/// The app's transcription preferences (stored in `library.db`, edited in « Réglages →
/// Transcription »). Per-project choices (language, vocabulary) live in the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionPreferences {
    /// The model to use (a [`TranscriptionModelInfo::id`]); `null` uses the best installed one
    /// for this machine.
    pub model: Option<String>,
    /// Whether to use the GPU when one is available.
    pub use_gpu: bool,
    /// Whether media are transcribed when they are imported.
    pub auto_transcribe: bool,
    /// Whether speech is detected first (voice model): fewer invented words over silences or
    /// music, faster, and word timings bounded by speech.
    pub voice_detection: bool,
    /// A folder holding the user's own `whisper-cli` build, searched first; `null` uses the
    /// bundled one (and the GPU pack when installed).
    pub whisper_dir: Option<String>,
}

impl Default for TranscriptionPreferences {
    fn default() -> Self {
        Self {
            model: None,
            use_gpu: true,
            auto_transcribe: false,
            voice_detection: true,
            whisper_dir: None,
        }
    }
}

/// A download or installation in progress (or failed) of a model or of the GPU pack.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DownloadProgress {
    /// Receiving bytes.
    Downloading {
        /// Bytes received so far.
        received: u64,
        /// Bytes expected.
        total: u64,
    },
    /// Checking the SHA-256 or unpacking the archive.
    Verifying,
    /// It failed; nothing was installed.
    Failed {
        /// Why, and what to do.
        message: String,
    },
}

/// What a download is about.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DownloadTarget {
    /// A transcription model of the catalogue.
    Model {
        /// Its id (`base`, `large-v3-turbo`…).
        model_id: String,
    },
    /// The voice detection model.
    VoiceModel,
    /// The GPU pack of whisper.cpp (Windows: CUDA build).
    GpuRuntime,
}

/// A transcription model: one of the catalogue (installed or not) or a file the user imported.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionModelInfo {
    /// Stable id: the catalogue's (`tiny`, `base`, `small`, `medium`, `large-v3-turbo`) or, for an
    /// imported model, its file name.
    pub id: String,
    /// The file in the models folder.
    pub file_name: String,
    /// Its size in bytes (expected for models not installed).
    pub size: u64,
    /// Whether the file is in the models folder.
    pub installed: bool,
    /// Whether it was imported (not one of the catalogue).
    pub custom: bool,
    /// Whether it is the catalogue model suggested for this machine.
    pub recommended: bool,
    /// Its download, while one runs or after one failed.
    pub download: Option<DownloadProgress>,
}

/// The voice detection model (one file, downloaded with the first transcription model).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct VoiceModelInfo {
    /// Its size in bytes.
    pub size: u64,
    /// Whether it is installed.
    pub installed: bool,
    /// Its download, while one runs or after one failed.
    pub download: Option<DownloadProgress>,
}

/// The whisper.cpp program found (or why none).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct WhisperInfo {
    /// Absolute path of the `whisper-cli` used, when found.
    pub path: Option<String>,
    /// What is wrong, when it cannot run (not found, missing system libraries).
    pub problem: Option<String>,
}

/// The GPU pack this platform can install (Windows: the CUDA build of whisper.cpp).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct GpuRuntimeInfo {
    /// What it is (`CUDA 12.4`).
    pub name: String,
    /// Download size in bytes.
    pub download_size: u64,
    /// Whether it is installed.
    pub installed: bool,
    /// Its download, while one runs or after one failed.
    pub download: Option<DownloadProgress>,
}

/// Everything « Réglages → Transcription » shows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionStatus {
    /// The program that transcribes.
    pub whisper: WhisperInfo,
    /// The GPU detected (`NVIDIA GeForce RTX 3080`, `Apple M2`), if any.
    pub gpu_name: Option<String>,
    /// Where the next transcription should run.
    pub backend: TranscriptionBackend,
    /// Where the last transcription of this session ran.
    pub last_backend: Option<TranscriptionBackend>,
    /// The GPU pack this platform offers, if any.
    pub gpu_runtime: Option<GpuRuntimeInfo>,
    /// The catalogue models, then imported ones.
    pub models: Vec<TranscriptionModelInfo>,
    /// The voice detection model.
    pub voice_model: VoiceModelInfo,
    /// Absolute path of the models folder (`<app data>/models/whisper`).
    pub models_dir: String,
    /// The preferences.
    pub preferences: TranscriptionPreferences,
}

/// A transcript segment matching a search, in its asset's time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptSearchHit {
    /// The asset.
    pub asset_id: AssetId,
    /// The segment's index in its transcript.
    pub segment: u32,
    /// Its start in the asset.
    pub start: Flicks,
    /// Its end in the asset.
    pub end: Flicks,
    /// Its text.
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn targets_and_progress_are_tagged() {
        let target = DownloadTarget::Model {
            model_id: "base".to_owned(),
        };
        assert_eq!(
            serde_json::to_value(&target).unwrap(),
            serde_json::json!({ "kind": "model", "modelId": "base" })
        );
        let progress = DownloadProgress::Downloading {
            received: 10,
            total: 20,
        };
        assert_eq!(
            serde_json::to_value(&progress).unwrap(),
            serde_json::json!({ "kind": "downloading", "received": 10, "total": 20 })
        );
        assert_eq!(
            serde_json::to_value(TranscriptionBackend::Cuda).unwrap(),
            serde_json::json!("cuda")
        );
    }

    #[test]
    fn default_preferences_use_the_gpu_and_voice_detection_without_auto_transcription() {
        let preferences = TranscriptionPreferences::default();
        assert!(preferences.use_gpu && preferences.voice_detection);
        assert!(!preferences.auto_transcribe);
        assert_eq!(preferences.model, None);
    }
}

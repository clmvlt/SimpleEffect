//! The transcription preferences, kept as `library.db` settings (`transcription.*`).

use se_protocol::TranscriptionPreferences;

use crate::{AppError, ports::AppLibraryRepository};

/// Keys of the transcription preferences in `library.db`.
pub mod keys {
    /// Id of the chosen model (empty: the best installed one).
    pub const MODEL: &str = "transcription.model";
    /// Use the GPU when available (`true` or `false`).
    pub const USE_GPU: &str = "transcription.useGpu";
    /// Transcribe imported media (`true` or `false`).
    pub const AUTO_TRANSCRIBE: &str = "transcription.autoTranscribe";
    /// Detect speech first (`true` or `false`).
    pub const VOICE_DETECTION: &str = "transcription.voiceDetection";
    /// Folder holding the user's own `whisper-cli` (empty: bundled builds).
    pub const WHISPER_DIR: &str = "transcription.whisperDir";
}

/// The stored preferences (defaults for values never set or unreadable).
///
/// # Errors
/// [`AppError::Storage`] when `library.db` cannot be read.
pub async fn load(
    library: &dyn AppLibraryRepository,
) -> Result<TranscriptionPreferences, AppError> {
    let defaults = TranscriptionPreferences::default();
    Ok(TranscriptionPreferences {
        model: text(library, keys::MODEL).await?,
        use_gpu: flag(library, keys::USE_GPU)
            .await?
            .unwrap_or(defaults.use_gpu),
        auto_transcribe: flag(library, keys::AUTO_TRANSCRIBE)
            .await?
            .unwrap_or(defaults.auto_transcribe),
        voice_detection: flag(library, keys::VOICE_DETECTION)
            .await?
            .unwrap_or(defaults.voice_detection),
        whisper_dir: text(library, keys::WHISPER_DIR).await?,
    })
}

/// Saves `preferences` (empty strings for `null`).
///
/// # Errors
/// [`AppError::InvalidSetting`] for a relative whisper folder; [`AppError::Storage`].
pub async fn save(
    library: &dyn AppLibraryRepository,
    preferences: &TranscriptionPreferences,
) -> Result<(), AppError> {
    let whisper_dir = preferences
        .whisper_dir
        .as_deref()
        .filter(|dir| !dir.is_empty());
    if whisper_dir.is_some_and(|dir| !std::path::Path::new(dir).is_absolute()) {
        return Err(AppError::InvalidSetting {
            name: "whisperDir",
            reason: "choose an absolute folder, or none to use the bundled whisper.cpp".to_owned(),
        });
    }
    let entries = [
        (keys::MODEL, preferences.model.clone().unwrap_or_default()),
        (keys::USE_GPU, preferences.use_gpu.to_string()),
        (
            keys::AUTO_TRANSCRIBE,
            preferences.auto_transcribe.to_string(),
        ),
        (
            keys::VOICE_DETECTION,
            preferences.voice_detection.to_string(),
        ),
        (
            keys::WHISPER_DIR,
            whisper_dir.unwrap_or_default().to_owned(),
        ),
    ];
    for (key, value) in entries {
        library.set_setting(key, &value).await?;
    }
    Ok(())
}

async fn text(library: &dyn AppLibraryRepository, key: &str) -> Result<Option<String>, AppError> {
    Ok(library
        .setting(key)
        .await?
        .filter(|value| !value.is_empty()))
}

async fn flag(library: &dyn AppLibraryRepository, key: &str) -> Result<Option<bool>, AppError> {
    Ok(match library.setting(key).await?.as_deref() {
        Some("true") => Some(true),
        Some("false") => Some(false),
        _ => None,
    })
}

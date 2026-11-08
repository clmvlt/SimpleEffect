//! The models folder seen through the catalogue: which models are installed, which one a
//! transcription uses, imports and removals. Decisions are pure; files go through
//! [`ModelFiles`].

use std::path::{Path, PathBuf};

use se_protocol::{DownloadProgress, TranscriptionModelInfo, TranscriptionPreferences};

use super::catalogue::{self, Hardware, VOICE_MODEL};
use crate::{
    AppError,
    ports::{ModelChoice, ModelFile, ModelFiles},
};

/// What the models folder holds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InstalledModels {
    /// The complete `.bin` files.
    pub files: Vec<ModelFile>,
}

impl InstalledModels {
    /// Whether a catalogue file is present with its expected size.
    #[must_use]
    pub fn has(&self, file: &catalogue::CatalogueFile) -> bool {
        self.files
            .iter()
            .any(|entry| entry.file_name == file.file_name && entry.size == file.size)
    }

    /// Imported models: `.bin` files that are not catalogue files.
    pub fn custom(&self) -> impl Iterator<Item = &ModelFile> {
        self.files
            .iter()
            .filter(|entry| !catalogue::is_catalogue_file(&entry.file_name))
    }

    /// Whether the voice detection model is installed.
    #[must_use]
    pub fn has_voice_model(&self) -> bool {
        self.has(&VOICE_MODEL)
    }

    /// The model a transcription uses: the chosen one when installed, else the best installed
    /// catalogue model for `hardware`; `None` when nothing fits.
    #[must_use]
    pub fn choose(
        &self,
        chosen: Option<&str>,
        hardware: Hardware,
        models_dir: &Path,
    ) -> Option<ModelChoice> {
        if let Some(id) = chosen
            && let Some(choice) = self.choice_of(id, models_dir)
        {
            return Some(choice);
        }
        catalogue::preference_order(hardware)
            .into_iter()
            .find_map(|id| self.choice_of(id, models_dir))
    }

    /// The installed model `id` (catalogue id or custom file name).
    fn choice_of(&self, id: &str, models_dir: &Path) -> Option<ModelChoice> {
        if let Some(model) = catalogue::model(id) {
            return self.has(&model.file).then(|| ModelChoice {
                id: model.id.to_owned(),
                path: models_dir.join(model.file.file_name),
                alignment: Some(model.alignment.to_owned()),
            });
        }
        self.custom()
            .find(|entry| entry.file_name == id)
            .map(|entry| ModelChoice {
                id: entry.file_name.clone(),
                path: models_dir.join(&entry.file_name),
                alignment: None,
            })
    }

    /// The catalogue models (installed or not), then the imported ones, for the settings.
    #[must_use]
    pub fn infos(
        &self,
        recommended: &str,
        download_of: impl Fn(&str) -> Option<DownloadProgress>,
    ) -> Vec<TranscriptionModelInfo> {
        let catalogue = catalogue::MODELS
            .iter()
            .map(|model| TranscriptionModelInfo {
                id: model.id.to_owned(),
                file_name: model.file.file_name.to_owned(),
                size: model.file.size,
                installed: self.has(&model.file),
                custom: false,
                recommended: model.id == recommended,
                download: download_of(model.id),
            });
        let custom = self.custom().map(|entry| TranscriptionModelInfo {
            id: entry.file_name.clone(),
            file_name: entry.file_name.clone(),
            size: entry.size,
            installed: true,
            custom: true,
            recommended: false,
            download: None,
        });
        catalogue.chain(custom).collect()
    }
}

/// Where an imported file goes and under which id: a catalogue model when its SHA-256 matches
/// one, else a custom model named after the file (`custom-` prefixed when it would take a
/// catalogue name, `.bin` appended when missing).
#[must_use]
pub fn import_target(source: &Path, sha256: &str) -> (String, String) {
    if let Some(model) = catalogue::model_with_sha256(sha256) {
        return (model.id.to_owned(), model.file.file_name.to_owned());
    }
    let name = source.file_name().map_or_else(
        || "model.bin".to_owned(),
        |name| name.to_string_lossy().into_owned(),
    );
    let name = if name.to_ascii_lowercase().ends_with(".bin") {
        name
    } else {
        format!("{name}.bin")
    };
    let name = if catalogue::is_catalogue_file(&name) {
        format!("custom-{name}")
    } else {
        name
    };
    (name.clone(), name)
}

/// Copies a model file into `models_dir` after checking it is a whisper.cpp model; returns its id.
///
/// # Errors
/// [`AppError::InvalidModelFile`]; media errors of the copy.
pub async fn import(
    files: &dyn ModelFiles,
    models_dir: &Path,
    source: &Path,
) -> Result<String, AppError> {
    if !files.is_whisper_model(source).await {
        return Err(AppError::InvalidModelFile {
            path: source.display().to_string(),
        });
    }
    let sha256 = files.sha256(source).await?;
    let (id, file_name) = import_target(source, &sha256);
    files.copy(source, &models_dir.join(file_name)).await?;
    Ok(id)
}

/// The file of model `id` in `models_dir` (catalogue or custom).
///
/// # Errors
/// [`AppError::TranscriptionTargetNotFound`] for an unknown or absent model.
pub fn installed_file(
    installed: &InstalledModels,
    models_dir: &Path,
    id: &str,
) -> Result<PathBuf, AppError> {
    let file_name = match catalogue::model(id) {
        Some(model) => model.file.file_name.to_owned(),
        None => id.to_owned(),
    };
    installed
        .files
        .iter()
        .any(|entry| entry.file_name == file_name)
        .then(|| models_dir.join(&file_name))
        .ok_or_else(|| AppError::TranscriptionTargetNotFound {
            what: format!("the model '{id}'"),
        })
}

/// The preferences after model `id` was removed (a removed choice falls back to automatic).
#[must_use]
pub fn without_model(
    mut preferences: TranscriptionPreferences,
    id: &str,
) -> TranscriptionPreferences {
    if preferences.model.as_deref() == Some(id) {
        preferences.model = None;
    }
    preferences
}

#[cfg(test)]
mod tests {
    use super::*;

    fn installed(names: &[(&str, u64)]) -> InstalledModels {
        InstalledModels {
            files: names
                .iter()
                .map(|(name, size)| ModelFile {
                    file_name: (*name).to_owned(),
                    size: *size,
                })
                .collect(),
        }
    }

    const GPU: Hardware = Hardware {
        gpu: true,
        cpu_threads: 16,
    };

    #[test]
    fn chooses_the_chosen_model_then_the_best_installed_one() {
        let dir = Path::new("/models");
        let models = installed(&[
            ("ggml-base.bin", 147_951_465),
            ("ggml-small.bin", 12),
            ("french.bin", 99),
        ]);
        let automatic = models.choose(None, GPU, dir).unwrap();
        assert_eq!(
            automatic.id, "base",
            "small has the wrong size: not installed"
        );
        assert_eq!(automatic.alignment.as_deref(), Some("base"));
        let custom = models.choose(Some("french.bin"), GPU, dir).unwrap();
        assert_eq!((custom.id.as_str(), custom.alignment), ("french.bin", None));
        assert_eq!(custom.path, dir.join("french.bin"));
        let gone = models.choose(Some("medium"), GPU, dir).unwrap();
        assert_eq!(gone.id, "base");
        assert_eq!(installed(&[]).choose(None, GPU, dir), None);
    }

    #[test]
    fn imports_catalogue_files_under_their_name_and_others_as_custom() {
        let base = catalogue::model("base").unwrap();
        assert_eq!(
            import_target(Path::new("/dl/whatever.bin"), base.file.sha256),
            ("base".to_owned(), "ggml-base.bin".to_owned())
        );
        assert_eq!(
            import_target(Path::new("/dl/french-distil"), "00"),
            (
                "french-distil.bin".to_owned(),
                "french-distil.bin".to_owned()
            )
        );
        assert_eq!(
            import_target(Path::new("/dl/ggml-base.bin"), "00").0,
            "custom-ggml-base.bin"
        );
    }

    #[test]
    fn lists_catalogue_then_custom_models_with_downloads() {
        let models = installed(&[("ggml-tiny.bin", 77_691_713), ("mine.bin", 5)]);
        let infos = models.infos("small", |id| {
            (id == "base").then_some(DownloadProgress::Verifying)
        });
        assert_eq!(infos.len(), 6);
        assert!(infos[0].installed && !infos[1].installed);
        assert_eq!(infos[1].download, Some(DownloadProgress::Verifying));
        assert!(infos[2].recommended);
        assert!(infos[5].custom && infos[5].installed);
        let preferences = TranscriptionPreferences {
            model: Some("mine.bin".to_owned()),
            ..TranscriptionPreferences::default()
        };
        assert_eq!(without_model(preferences, "mine.bin").model, None);
    }
}

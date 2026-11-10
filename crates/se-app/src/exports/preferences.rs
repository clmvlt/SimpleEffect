//! What the export dialog remembers, kept as `library.db` settings (`export.*`), and its checks.

use std::path::Path;

use se_protocol::{ExportPreferences, ExportPreset, ExportSettings, VideoExportSettings};

use super::settings::check_settings;
use crate::{AppError, ports::AppLibraryRepository};

/// Keys of the export preferences in `library.db`.
pub mod keys {
    /// The folder of the last export.
    pub const DIRECTORY: &str = "export.directory";
    /// The file name template.
    pub const FILE_NAME_TEMPLATE: &str = "export.fileNameTemplate";
    /// The saved presets (JSON).
    pub const PRESETS: &str = "export.presets";
}

/// The template of new installations: the sequence's name.
pub const DEFAULT_FILE_NAME_TEMPLATE: &str = "{sequence}";
/// Most saved presets.
const MAX_PRESETS: usize = 50;
/// Longest preset name and id, in characters.
const MAX_PRESET_NAME_CHARS: usize = 60;
const MAX_PRESET_ID_CHARS: usize = 64;
/// Longest file name template, in characters.
const MAX_TEMPLATE_CHARS: usize = 200;
/// Characters no file name may hold on any of the three systems.
const FORBIDDEN_FILE_NAME_CHARS: &[char] = &['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// The stored preferences; `default_directory` (the user's videos folder) until one is set, and
/// defaults for values never set or unreadable.
///
/// # Errors
/// [`AppError::Storage`] when `library.db` cannot be read.
pub async fn load(
    library: &dyn AppLibraryRepository,
    default_directory: &Path,
) -> Result<ExportPreferences, AppError> {
    let directory = library
        .setting(keys::DIRECTORY)
        .await?
        .filter(|directory| Path::new(directory).is_absolute())
        .unwrap_or_else(|| default_directory.to_string_lossy().into_owned());
    let file_name_template = library
        .setting(keys::FILE_NAME_TEMPLATE)
        .await?
        .filter(|template| check_template(template).is_ok())
        .unwrap_or_else(|| DEFAULT_FILE_NAME_TEMPLATE.to_owned());
    let presets = library
        .setting(keys::PRESETS)
        .await?
        .and_then(|text| ExportPreset::list_from_json(&text))
        .filter(|presets| check_presets(presets).is_ok())
        .unwrap_or_default();
    Ok(ExportPreferences {
        directory,
        file_name_template,
        presets,
    })
}

/// Stores `preferences` once checked.
///
/// # Errors
/// [`AppError::InvalidExport`] naming the faulty value; [`AppError::Storage`] when `library.db`
/// cannot be written.
pub async fn save(
    library: &dyn AppLibraryRepository,
    preferences: &ExportPreferences,
) -> Result<(), AppError> {
    check(preferences).map_err(|reason| AppError::InvalidExport { reason })?;
    let presets = ExportPreset::list_to_json(&preferences.presets)
        .ok_or_else(|| AppError::Internal("export presets could not be serialized".to_owned()))?;
    library
        .set_setting(keys::DIRECTORY, &preferences.directory)
        .await?;
    library
        .set_setting(keys::FILE_NAME_TEMPLATE, &preferences.file_name_template)
        .await?;
    library.set_setting(keys::PRESETS, &presets).await?;
    Ok(())
}

/// Checks every value of `preferences`.
///
/// # Errors
/// A message naming the faulty value.
pub fn check(preferences: &ExportPreferences) -> Result<(), String> {
    if !Path::new(&preferences.directory).is_absolute() {
        return Err(format!(
            "the export folder must be an absolute path, not \"{}\"",
            preferences.directory
        ));
    }
    check_template(&preferences.file_name_template)?;
    check_presets(&preferences.presets)
}

fn check_template(template: &str) -> Result<(), String> {
    let length = template.chars().count();
    if template.trim().is_empty() || length > MAX_TEMPLATE_CHARS {
        return Err(format!(
            "the file name template must hold 1 to {MAX_TEMPLATE_CHARS} characters"
        ));
    }
    if let Some(found) = template
        .chars()
        .find(|character| FORBIDDEN_FILE_NAME_CHARS.contains(character) || character.is_control())
    {
        return Err(format!(
            "the file name template cannot contain \"{found}\" (not allowed in file names)"
        ));
    }
    Ok(())
}

fn check_presets(presets: &[ExportPreset]) -> Result<(), String> {
    if presets.len() > MAX_PRESETS {
        return Err(format!("at most {MAX_PRESETS} presets can be saved"));
    }
    for (index, preset) in presets.iter().enumerate() {
        check_preset(preset).map_err(|reason| format!("preset \"{}\": {reason}", preset.name))?;
        if presets[..index].iter().any(|other| other.id == preset.id) {
            return Err(format!("two presets share the id \"{}\"", preset.id));
        }
    }
    Ok(())
}

fn check_preset(preset: &ExportPreset) -> Result<(), String> {
    let name = preset.name.trim().chars().count();
    if name == 0 || name > MAX_PRESET_NAME_CHARS {
        return Err(format!(
            "a preset name holds 1 to {MAX_PRESET_NAME_CHARS} characters"
        ));
    }
    if preset.id.is_empty() || preset.id.chars().count() > MAX_PRESET_ID_CHARS {
        return Err(format!(
            "a preset id holds 1 to {MAX_PRESET_ID_CHARS} characters"
        ));
    }
    // A preset fixes the shorter side; a square frame of that side checks its codec choices.
    let video = preset.video.map(|video| VideoExportSettings {
        codec: video.codec,
        width: video.short_side,
        height: video.short_side,
        rate: video.rate,
    });
    check_settings(&ExportSettings {
        container: preset.container,
        video,
        audio: preset.audio,
    })
}

#[cfg(test)]
mod tests {
    use se_protocol::{
        AudioEncoderChoice, AudioExportCodec, AudioExportSettings, ExportContainer,
        ExportPresetVideo, ExportQuality, VideoExportCodec, VideoRate,
    };

    use super::*;

    fn preset(id: &str, name: &str) -> ExportPreset {
        ExportPreset {
            id: id.to_owned(),
            name: name.to_owned(),
            container: ExportContainer::Mp4,
            video: Some(ExportPresetVideo {
                codec: VideoExportCodec::H264,
                short_side: 1080,
                rate: VideoRate::Quality {
                    level: ExportQuality::High,
                },
            }),
            audio: Some(AudioExportSettings {
                codec: AudioExportCodec::Aac,
                bits_per_second: 192_000,
                encoder: AudioEncoderChoice::WebCodecs,
            }),
        }
    }

    fn preferences() -> ExportPreferences {
        let directory = if cfg!(windows) {
            "C:\\Users\\me\\Videos"
        } else {
            "/home/me/Videos"
        };
        ExportPreferences {
            directory: directory.to_owned(),
            file_name_template: "{project} - {sequence}".to_owned(),
            presets: vec![preset("a", "YouTube maison")],
        }
    }

    #[test]
    fn valid_preferences_pass() {
        assert_eq!(check(&preferences()), Ok(()));
    }

    #[test]
    fn faulty_values_are_named() {
        let mut relative = preferences();
        relative.directory = "Videos".to_owned();
        assert!(check(&relative).unwrap_err().contains("absolute"));
        let mut slash = preferences();
        slash.file_name_template = "{sequence}/final".to_owned();
        assert!(check(&slash).unwrap_err().contains("\"/\""));
        let mut twice = preferences();
        twice.presets.push(preset("a", "Autre"));
        assert!(check(&twice).unwrap_err().contains("share the id"));
        let mut unnamed = preferences();
        unnamed.presets = vec![preset("b", "  ")];
        assert!(check(&unnamed).unwrap_err().contains("preset name"));
        let mut odd = preferences();
        odd.presets = vec![ExportPreset {
            video: Some(ExportPresetVideo {
                codec: VideoExportCodec::Vp9,
                short_side: 1080,
                rate: VideoRate::Quality {
                    level: ExportQuality::High,
                },
            }),
            ..preset("c", "VP9 en MP4")
        }];
        assert!(check(&odd).unwrap_err().contains("VP9 en MP4"));
    }

    #[test]
    fn presets_round_trip_through_json() {
        let presets = preferences().presets;
        let text = ExportPreset::list_to_json(&presets).unwrap();
        assert_eq!(ExportPreset::list_from_json(&text), Some(presets));
        assert_eq!(ExportPreset::list_from_json("not json"), None);
    }
}

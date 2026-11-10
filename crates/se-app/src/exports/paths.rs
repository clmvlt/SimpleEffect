//! The file of an export: where it may be written.

use std::path::PathBuf;

use se_protocol::ExportSettings;

use super::settings;

/// The export's file once checked: absolute, with the container's extension.
pub(super) fn checked_path(path: &str, settings: &ExportSettings) -> Result<PathBuf, String> {
    let path = PathBuf::from(path);
    if !path.is_absolute() {
        return Err(format!(
            "the export file must be an absolute path, not \"{}\"",
            path.display()
        ));
    }
    let expected = settings::extension_of(settings.container);
    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);
    if extension.as_deref() != Some(expected) {
        return Err(format!(
            "the export file must end with .{expected} for this format: {}",
            path.display()
        ));
    }
    if path.file_stem().is_none_or(std::ffi::OsStr::is_empty) {
        return Err("the export file needs a name".to_owned());
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use se_protocol::{AudioEncoderChoice, AudioExportCodec, AudioExportSettings, ExportContainer};

    use super::*;

    fn wav() -> ExportSettings {
        ExportSettings {
            container: ExportContainer::Wav,
            video: None,
            audio: Some(AudioExportSettings {
                codec: AudioExportCodec::Pcm,
                bits_per_second: 1_536_000,
                encoder: AudioEncoderChoice::WebCodecs,
            }),
        }
    }

    fn absolute(name: &str) -> String {
        if cfg!(windows) {
            format!("C:\\exports\\{name}")
        } else {
            format!("/exports/{name}")
        }
    }

    #[test]
    fn paths_need_an_absolute_name_with_the_right_extension() {
        assert!(checked_path(&absolute("son.wav"), &wav()).is_ok());
        assert!(checked_path(&absolute("SON.WAV"), &wav()).is_ok());
        assert!(
            checked_path("son.wav", &wav())
                .unwrap_err()
                .contains("absolute")
        );
        assert!(
            checked_path(&absolute("son.mp4"), &wav())
                .unwrap_err()
                .contains(".wav")
        );
        assert!(checked_path(&absolute(".wav"), &wav()).is_err());
    }
}

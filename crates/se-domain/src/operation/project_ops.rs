//! Project operations: [`RenameProject`], [`SetTranscriptionSettings`].

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Edit, Effect, checks::ensure_name};
use crate::{DomainError, Project, TranscriptionSettings};

/// Renames the project (its folder is renamed by persistence, prompt 07).
/// Inverse: [`RenameProject`] with the previous name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct RenameProject {
    /// The new name (not empty).
    pub name: String,
}

impl Edit for RenameProject {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        ensure_name("project", &self.name)?;
        let previous = std::mem::replace(&mut project.name, self.name.clone());
        effect.touched.project = true;
        effect.undo_with(Self { name: previous });
        Ok(())
    }
}

/// Sets how the project's media are transcribed (language, vocabulary).
/// Inverse: [`SetTranscriptionSettings`] with the previous settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SetTranscriptionSettings {
    /// The new settings (checked by [`TranscriptionSettings::check`]).
    pub settings: TranscriptionSettings,
}

impl Edit for SetTranscriptionSettings {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        self.settings.check()?;
        let previous =
            std::mem::replace(&mut project.settings.transcription, self.settings.clone());
        effect.touched.project = true;
        effect.undo_with(Self { settings: previous });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::operation::testing::{apply_err, apply_ok, example};

    #[test]
    fn renames_with_a_non_empty_name() {
        let applied = apply_ok(
            &example(),
            RenameProject {
                name: "Interview".to_owned(),
            },
        );
        assert_eq!(applied.project.name, "Interview");
        assert!(applied.touched.project);
        assert_eq!(
            apply_err(
                &example(),
                RenameProject {
                    name: "  ".to_owned()
                }
            ),
            DomainError::EmptyName { entity: "project" }
        );
    }

    #[test]
    fn sets_checked_transcription_settings() {
        let settings = TranscriptionSettings {
            language: None,
            vocabulary: "SimpleEffect".to_owned(),
        };
        let applied = apply_ok(
            &example(),
            SetTranscriptionSettings {
                settings: settings.clone(),
            },
        );
        assert_eq!(applied.project.settings.transcription, settings);
        assert!(applied.touched.project);
        let refused = apply_err(
            &example(),
            SetTranscriptionSettings {
                settings: TranscriptionSettings {
                    language: Some("French".to_owned()),
                    vocabulary: String::new(),
                },
            },
        );
        assert!(matches!(
            refused,
            DomainError::InvalidTranscriptionSettings { .. }
        ));
    }
}

//! Project operations: [`RenameProject`], [`SetTranscriptionSettings`], [`SetBrandKit`].

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Edit, Effect, checks::ensure_name};
use crate::{BrandKit, DomainError, Project, TranscriptionSettings};

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

/// Sets the project's brand kit (a copy of a library kit, or none). Logos must already be
/// assets of the project (validation checks them at the end of the transaction).
/// Inverse: [`SetBrandKit`] with the previous kit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SetBrandKit {
    /// The new kit (`null` removes it; checked by [`BrandKit::problems`]).
    pub brand_kit: Option<BrandKit>,
}

impl Edit for SetBrandKit {
    fn apply_to(&self, project: &mut Project, effect: &mut Effect) -> Result<(), DomainError> {
        if let Some(kit) = &self.brand_kit {
            let problems = kit.problems();
            if !problems.is_empty() {
                return Err(DomainError::InvalidBrandKit {
                    name: kit.name.clone(),
                    problems: problems
                        .into_iter()
                        .map(|(field, rule)| format!("{field} {rule}"))
                        .collect::<Vec<_>>()
                        .join("; "),
                });
            }
        }
        let previous = std::mem::replace(&mut project.brand_kit, self.brand_kit.clone());
        effect.touched.project = true;
        effect.undo_with(Self {
            brand_kit: previous,
        });
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

    #[test]
    fn sets_and_removes_the_brand_kit() {
        let kit = example().brand_kit.unwrap();
        let applied = apply_ok(&example(), SetBrandKit { brand_kit: None });
        assert_eq!(applied.project.brand_kit, None);
        assert!(applied.touched.project);
        let mut renamed = kit.clone();
        renamed.name = "Acme".to_owned();
        let applied = apply_ok(
            &applied.project,
            SetBrandKit {
                brand_kit: Some(renamed),
            },
        );
        assert_eq!(applied.project.brand_kit.unwrap().name, "Acme");
        let mut broken = kit;
        broken.fonts.heading = " ".to_owned();
        let refused = apply_err(
            &example(),
            SetBrandKit {
                brand_kit: Some(broken),
            },
        );
        assert_eq!(refused.code(), crate::DomainErrorCode::InvalidBrandKit);
        assert!(refused.to_string().contains("fonts.heading"), "{refused}");
    }
}

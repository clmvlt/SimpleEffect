//! [`TranscriptionSettings`]: how a project's media are transcribed (language, vocabulary).

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::DomainError;

/// How the media of a project are transcribed (prompt 21). Changed by
/// [`crate::SetTranscriptionSettings`] (a user edit, undoable).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TranscriptionSettings {
    /// The spoken language as an ISO 639 code (`fr`, `en`, `yue`…), or `None` to let the model
    /// detect it for each media file.
    pub language: Option<String>,
    /// Names and terms the model should spell this way (people, brands, jargon), as one line of
    /// text given to it as context; empty for none.
    pub vocabulary: String,
}

impl TranscriptionSettings {
    /// The default language of new projects: the app is French first.
    pub const DEFAULT_LANGUAGE: &'static str = "fr";

    /// The longest vocabulary, in characters: the model reads at most about 220 tokens of context.
    pub const MAX_VOCABULARY_CHARS: usize = 600;

    /// Checks the language code and the vocabulary.
    ///
    /// # Errors
    /// [`DomainError::InvalidTranscriptionSettings`] for a language that is not 2–3 lowercase
    /// letters, or a vocabulary that is too long or holds line breaks or control characters.
    pub fn check(&self) -> Result<(), DomainError> {
        if let Some(language) = &self.language {
            let letters = language.chars().all(|c| c.is_ascii_lowercase());
            if !letters || !(2..=3).contains(&language.len()) {
                return Err(invalid(format!(
                    "the language '{language}' is not an ISO 639 code: use two or three lowercase \
                     letters such as 'fr' or 'en', or null to detect it"
                )));
            }
        }
        let length = self.vocabulary.chars().count();
        if length > Self::MAX_VOCABULARY_CHARS {
            return Err(invalid(format!(
                "the vocabulary has {length} characters; keep it under {} (the most important \
                 names first)",
                Self::MAX_VOCABULARY_CHARS
            )));
        }
        if self.vocabulary.chars().any(char::is_control) {
            return Err(invalid(
                "the vocabulary must be one line of text: separate names with commas, not line \
                 breaks"
                    .to_owned(),
            ));
        }
        Ok(())
    }
}

impl Default for TranscriptionSettings {
    fn default() -> Self {
        Self {
            language: Some(Self::DEFAULT_LANGUAGE.to_owned()),
            vocabulary: String::new(),
        }
    }
}

fn invalid(reason: String) -> DomainError {
    DomainError::InvalidTranscriptionSettings { reason }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(language: Option<&str>, vocabulary: &str) -> TranscriptionSettings {
        TranscriptionSettings {
            language: language.map(str::to_owned),
            vocabulary: vocabulary.to_owned(),
        }
    }

    #[test]
    fn accepts_iso_codes_detection_and_a_one_line_vocabulary() {
        assert_eq!(TranscriptionSettings::default().check(), Ok(()));
        assert_eq!(settings(None, "").check(), Ok(()));
        assert_eq!(
            settings(Some("yue"), "SimpleEffect, Dimzou").check(),
            Ok(())
        );
    }

    #[test]
    fn refuses_bad_languages_and_vocabularies() {
        for language in ["FR", "f", "fran", "fr-FR", ""] {
            let error = settings(Some(language), "").check().unwrap_err();
            assert!(matches!(
                error,
                DomainError::InvalidTranscriptionSettings { .. }
            ));
        }
        let long = "a".repeat(TranscriptionSettings::MAX_VOCABULARY_CHARS + 1);
        assert!(settings(None, &long).check().is_err());
        assert!(settings(None, "Alice\nBob").check().is_err());
    }
}

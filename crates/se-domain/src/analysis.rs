//! Media analyses (`docs/architecture.md` §7): what the analysis jobs measured on an asset's sound
//! ([`AudioAnalysis`]) and pictures ([`SceneDetection`]).
//!
//! Full results live in the project database (`analyses` table, written by se-store), not in the
//! document: they can be large (one loudness value per half second) and are facts about a file's
//! content, not edits. The document keeps a small summary per asset ([`AssetAnalysis`]) that the
//! media bin shows and AI agents read with the asset.

mod audio;
mod scenes;

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::TranscriptSummary;

pub use self::{
    audio::{AudioAnalysis, AudioSummary, LoudnessCurve, SilenceSettings},
    scenes::{SceneDetection, SceneSummary},
};

/// Which analysis a stored result is (the `kind` column of the `analyses` table).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum AnalysisKind {
    /// Silences, loudness and peaks of the first sound stream.
    AudioAnalysis,
    /// Instants where the picture changes shot.
    SceneDetection,
}

impl AnalysisKind {
    /// The stable name stored in the database (`audioAnalysis`, `sceneDetection`).
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AudioAnalysis => "audioAnalysis",
            Self::SceneDetection => "sceneDetection",
        }
    }
}

/// One analysis result, tagged by its kind.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum AnalysisResult {
    /// Result of an audio analysis.
    AudioAnalysis(AudioAnalysis),
    /// Result of a scene detection.
    SceneDetection(SceneDetection),
}

impl AnalysisResult {
    /// Which analysis this is.
    #[must_use]
    pub const fn kind(&self) -> AnalysisKind {
        match self {
            Self::AudioAnalysis(_) => AnalysisKind::AudioAnalysis,
            Self::SceneDetection(_) => AnalysisKind::SceneDetection,
        }
    }
}

/// The analysis summaries of an asset, kept in the document. `None` until the analysis ran on
/// the asset's current content (relinking to another content clears them).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AssetAnalysis {
    /// Loudness, peaks and silences of the first sound stream.
    pub audio: Option<AudioSummary>,
    /// Shot changes of the picture.
    pub scenes: Option<SceneSummary>,
    /// The words spoken (prompt 21; schema version 7).
    #[serde(default)]
    pub transcript: Option<TranscriptSummary>,
}

/// `value` when it is a finite number: measurements of silence are `-∞` and JSON has no
/// infinities, so they are stored as `None`.
#[must_use]
pub fn finite(value: f64) -> Option<f64> {
    value.is_finite().then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Flicks;

    #[test]
    fn results_are_tagged_by_kind() {
        let result = AnalysisResult::SceneDetection(SceneDetection {
            threshold: 0.3,
            changes: vec![Flicks::one_second()],
        });
        assert_eq!(result.kind(), AnalysisKind::SceneDetection);
        assert_eq!(
            serde_json::to_string(&result).unwrap(),
            r#"{"kind":"sceneDetection","threshold":0.3,"changes":[705600000]}"#
        );
        assert_eq!(result.kind().as_str(), "sceneDetection");
    }

    #[test]
    fn infinities_become_none() {
        assert_eq!(finite(-12.5), Some(-12.5));
        assert_eq!(finite(f64::NEG_INFINITY), None);
        assert_eq!(finite(f64::NAN), None);
    }
}

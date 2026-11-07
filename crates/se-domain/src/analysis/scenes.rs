//! [`SceneDetection`]: the instants where an asset's picture changes shot.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::time::Flicks;

/// Full result of a scene detection (stored in the project database).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SceneDetection {
    /// Scene score (ffmpeg's `scene`, 0 to 1) above which a frame starts a new shot.
    pub threshold: f64,
    /// Source times of the first frame of each new shot, in order (the start of the media is not
    /// listed).
    pub changes: Vec<Flicks>,
}

impl SceneDetection {
    /// Default score threshold: catches cuts, ignores camera moves and flashes most of the time.
    pub const DEFAULT_THRESHOLD: f64 = 0.3;

    /// The summary kept in the document.
    #[must_use]
    pub fn summary(&self) -> SceneSummary {
        SceneSummary {
            change_count: u32::try_from(self.changes.len()).unwrap_or(u32::MAX),
        }
    }
}

/// The scene detection of an asset as the document keeps it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SceneSummary {
    /// Number of shot changes detected.
    pub change_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_summary_counts_changes() {
        let detection = SceneDetection {
            threshold: SceneDetection::DEFAULT_THRESHOLD,
            changes: vec![Flicks::one_second(), Flicks::from_millis(4_200)],
        };
        assert_eq!(detection.summary(), SceneSummary { change_count: 2 });
    }
}

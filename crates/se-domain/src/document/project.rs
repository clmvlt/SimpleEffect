//! [`Project`]: the root of the document.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Asset, BrandKit, Format, MotionTemplate, Sequence};
use crate::ids::ProjectId;

/// Version of the document format written by this build. Persistence (prompt 07) migrates older
/// documents up to it and refuses newer ones.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// The whole editable state of a project, serialized as `project.json` (ADR-0006).
///
/// Mutated only by the project actor applying operations (golden rule 4, prompt 06).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    /// Stable id.
    pub id: ProjectId,
    /// Name shown in the UI (also the default folder name).
    pub name: String,
    /// Document format version ([`CURRENT_SCHEMA_VERSION`] once loaded).
    pub schema_version: u32,
    /// Number of transactions committed so far; bumped by every commit.
    pub revision: u64,
    /// Project-wide preferences.
    pub settings: ProjectSettings,
    /// Imported media, in import order.
    pub assets: Vec<Asset>,
    /// Timelines, in creation order.
    pub sequences: Vec<Sequence>,
    /// Motion templates available to template items.
    pub templates: Vec<MotionTemplate>,
    /// The brand identity used by this project, if one was set.
    pub brand_kit: Option<BrandKit>,
}

/// Project-wide preferences.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {
    /// Format given to new sequences.
    pub default_format: Format,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::example_project;

    /// Fixes the document format: any change to this snapshot is a change of `project.json` and
    /// of the protocol, and may need a schema migration (prompt 07).
    #[test]
    fn example_project_json_format() {
        insta::assert_json_snapshot!(example_project());
    }

    #[test]
    fn round_trips_through_json() {
        let project = example_project();
        let json = serde_json::to_string(&project).unwrap();
        assert_eq!(serde_json::from_str::<Project>(&json).unwrap(), project);
    }
}

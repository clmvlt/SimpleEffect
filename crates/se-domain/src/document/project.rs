//! [`Project`]: the root of the document.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{
    Asset, BrandKit, CaptionStyle, Format, Item, MotionTemplate, Sequence, TranscriptionSettings,
};
use crate::ids::ProjectId;

/// Version of the document format written by this build. Persistence (prompt 07) migrates older
/// documents up to it and refuses newer ones.
///
/// Version 2 (prompt 08): assets carry a content fingerprint and a decodability per stream.
/// Version 3 (prompt 09): assets carry the summaries of their analyses (`analysis`).
/// Version 4 (prompt 14): tracks carry transitions, media clips fade shapes.
/// Version 5 (prompt 15): full text styles, outlines, image fits, blur, animatable colours.
/// Version 6 (prompt 16): media clips that fill the frame carry a focus (`reframe.focus`).
/// Version 7 (prompt 21): transcription settings and transcript summaries.
/// Version 8 (prompt 23): caption styles, captions made of timed words.
/// Version 9 (prompt 24): motion templates with source, parameters and versions; template items
/// with a transform and animatable values.
/// Version 10 (prompt 25): complete brand kits (copied from the library: fonts, named palette,
/// logos, default text and caption looks, favourite templates); templates carry a category and
/// their library link, parameters an optional brand role.
pub const CURRENT_SCHEMA_VERSION: u32 = 10;

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
    /// The brand identity used by this project, if one was set: a copy of a library kit
    /// (schema version 10), so the project behaves the same without the library.
    pub brand_kit: Option<BrandKit>,
    /// Looks shared by its captions (schema version 8).
    #[serde(default)]
    pub caption_styles: Vec<CaptionStyle>,
}

impl Project {
    /// Every item of every sequence, sequence by sequence, track by track.
    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.sequences
            .iter()
            .flat_map(|sequence| &sequence.tracks)
            .flat_map(|track| &track.items)
    }
}

/// Project-wide preferences.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSettings {
    /// Format given to new sequences.
    pub default_format: Format,
    /// How the project's media are transcribed (schema version 7).
    #[serde(default)]
    pub transcription: TranscriptionSettings,
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

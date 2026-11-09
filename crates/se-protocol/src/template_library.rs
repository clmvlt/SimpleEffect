//! The app's template library (prompt 25, ADR-0036): motion templates and caption styles saved
//! once and reused in every project, stored in `<app data>/templates/<id>/`.

use se_domain::{
    CaptionStyle, Flicks, ItemId, LibraryTemplateId, ProjectId, SequenceId, TemplateCategory,
    TemplateId, TemplateParam, TrackId,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// One entry of the template library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LibraryTemplate {
    /// Stable id (project copies link to it).
    pub id: LibraryTemplateId,
    /// Version of the content, 1 when saved first, bumped by each new content; project copies
    /// holding an older version are offered the update.
    pub version: u32,
    /// Name shown in the browser.
    pub name: String,
    /// What it shows (may be empty).
    pub description: String,
    /// Free words to search by.
    pub tags: Vec<String>,
    /// When it last changed (Unix milliseconds).
    pub updated_at: u64,
    /// The template itself.
    pub content: LibraryContent,
}

/// What a library entry holds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum LibraryContent {
    /// A motion template (checked by the renderer when it was saved or imported).
    MotionTemplate {
        /// Its browser category.
        category: TemplateCategory,
        /// The TypeScript module.
        source: String,
        /// The parameters it declares, in order.
        params: Vec<TemplateParam>,
        /// Length of new instances.
        default_duration: Flicks,
    },
    /// A caption look (its `id` is not used: applying it makes or changes project styles).
    CaptionStyle {
        /// The look.
        style: CaptionStyle,
    },
}

/// What « Enregistrer dans la bibliothèque » keeps of a project template or caption style.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LibraryTemplateInfo {
    /// Name shown in the browser (not empty).
    pub name: String,
    /// What it shows.
    pub description: String,
    /// Free words to search by.
    pub tags: Vec<String>,
    /// Browser category of a motion template (ignored for caption styles; `null` keeps the
    /// template's own).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub category: Option<TemplateCategory>,
}

/// Where a library template is inserted in a project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InsertLibraryTemplateRequest {
    /// The open project.
    pub project_id: ProjectId,
    /// The library entry (a motion template).
    pub template_id: LibraryTemplateId,
    /// The sequence.
    pub sequence_id: SequenceId,
    /// The video track (`null`: the first free video track at that time, or a new one).
    pub track_id: Option<TrackId>,
    /// Where the instance starts.
    pub start: Flicks,
}

/// What inserting a library template did.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct InsertedLibraryTemplate {
    /// The project's copy of the template (an existing copy of the same version is reused).
    pub template_id: TemplateId,
    /// The new instance.
    pub item_id: ItemId,
    /// Its track.
    pub track_id: TrackId,
}

/// What updating a project template from the library did.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct TemplateUpdateReport {
    /// The project template's new version.
    pub version: u32,
    /// The library version it now holds.
    pub library_version: u32,
    /// Instance values that no longer fitted and went back to the default, as
    /// `item id: parameter` pairs.
    pub reset: Vec<ResetValue>,
}

/// One instance value reset by a new version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ResetValue {
    /// The instance.
    pub item_id: ItemId,
    /// The parameter.
    pub param: String,
}

/// What importing one `.setemplate` file did.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum LibraryImportOutcome {
    /// A new entry.
    Imported {
        /// The entry.
        template: LibraryTemplate,
    },
    /// A newer version of an entry of the library replaced it.
    Updated {
        /// The entry.
        template: LibraryTemplate,
    },
    /// The library already holds this version (or a newer one): nothing changed.
    Unchanged {
        /// The library's entry.
        template: LibraryTemplate,
    },
    /// The file was refused.
    Failed {
        /// Why, and what to do.
        message: String,
    },
}

/// The outcome of one file of an import.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct LibraryImportResult {
    /// The file.
    pub path: String,
    /// What happened.
    pub outcome: LibraryImportOutcome,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn entries_are_tagged_by_content_kind() {
        let style = se_domain::caption_style_from_preset(
            "minimal",
            se_domain::CaptionStyleId::from_test_index(1),
        )
        .unwrap();
        let entry = LibraryTemplate {
            id: LibraryTemplateId::from_test_index(1),
            version: 2,
            name: "Sous-titres maison".to_owned(),
            description: String::new(),
            tags: vec!["shorts".to_owned()],
            updated_at: 1_700_000_000_000,
            content: LibraryContent::CaptionStyle { style },
        };
        let value = serde_json::to_value(&entry).unwrap();
        assert_eq!(value["content"]["kind"], "captionStyle");
        assert_eq!(value["updatedAt"], json!(1_700_000_000_000_u64));
        assert_eq!(
            serde_json::from_value::<LibraryTemplate>(value).unwrap(),
            entry
        );
    }

    #[test]
    fn import_outcomes_carry_their_kind() {
        let failed = LibraryImportResult {
            path: "/tmp/x.setemplate".to_owned(),
            outcome: LibraryImportOutcome::Failed {
                message: "not a zip".to_owned(),
            },
        };
        assert_eq!(
            serde_json::to_value(failed).unwrap(),
            json!({ "path": "/tmp/x.setemplate", "outcome": { "kind": "failed", "message": "not a zip" } })
        );
    }
}

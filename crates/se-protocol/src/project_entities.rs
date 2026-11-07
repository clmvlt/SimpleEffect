//! The open project as the renderer mirrors it: whole entities, normalised by id
//! (`docs/architecture.md` §5.6, ADR-0005).
//!
//! Children are referenced by id from their parent (`trackIds`, `itemIds`…) and point back to it
//! (`sequenceId`, `trackId`), so a patch replaces entities one by one and the renderer never
//! re-implements an operation. se-app builds these DTOs from the document.

use se_domain::{
    Asset, BrandKit, Format, Item, ItemId, Marker, MarkerId, MotionTemplate, ProjectId,
    ProjectSettings, SequenceId, TrackId, TrackKind, Transition,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Every entity of a project at one revision: what a client loads before applying patches.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectSnapshot {
    /// Revision of the document these entities describe; the next patch has `revision + 1`.
    pub revision: u64,
    /// The project's own fields.
    pub project: ProjectHeader,
    /// Sequences, in project order.
    pub sequences: Vec<SequenceEntity>,
    /// Every track of every sequence.
    pub tracks: Vec<TrackEntity>,
    /// Every item of every track.
    pub items: Vec<ItemEntity>,
    /// Every marker of every sequence.
    pub markers: Vec<MarkerEntity>,
    /// Imported media, in import order.
    pub assets: Vec<Asset>,
}

/// The fields of a project that are not entities of their own.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectHeader {
    /// Stable id.
    pub id: ProjectId,
    /// Name shown in the UI.
    pub name: String,
    /// Project-wide preferences.
    pub settings: ProjectSettings,
    /// Motion templates available to template items.
    pub templates: Vec<MotionTemplate>,
    /// The brand identity of the project, if one was set.
    pub brand_kit: Option<BrandKit>,
}

/// A sequence without its tracks and markers, which are entities of their own.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct SequenceEntity {
    /// Stable id.
    pub id: SequenceId,
    /// Name shown in the UI.
    pub name: String,
    /// Frame size and rates.
    pub format: Format,
    /// Its tracks, in stacking order (first video track = bottom layer).
    pub track_ids: Vec<TrackId>,
    /// Its markers, in document order.
    pub marker_ids: Vec<MarkerId>,
}

/// A track header and the ids of its items.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::struct_excessive_bools,
    reason = "the four flags are independent toggles of the track header"
)]
pub struct TrackEntity {
    /// Stable id.
    pub id: TrackId,
    /// The sequence holding the track.
    pub sequence_id: SequenceId,
    /// Video or audio.
    pub kind: TrackKind,
    /// Name shown in the track header.
    pub name: String,
    /// Audio of this track is silenced.
    pub muted: bool,
    /// Only solo tracks are heard when at least one track is solo.
    pub solo: bool,
    /// Items of this track cannot be edited.
    pub locked: bool,
    /// Pictures of this track are not drawn.
    pub hidden: bool,
    /// Track gain in decibels.
    pub volume_db: f64,
    /// Stereo balance from −1 (left) to 1 (right).
    pub pan: f64,
    /// Its items, sorted by start time.
    pub item_ids: Vec<ItemId>,
    /// Its cross-fades (small values, sent whole with the track: a transition change upserts
    /// its track).
    pub transitions: Vec<Transition>,
}

/// An item and the track holding it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ItemEntity {
    /// The track holding the item.
    pub track_id: TrackId,
    /// The item, its fields flattened next to `trackId`.
    #[serde(flatten)]
    pub item: Item,
}

/// A marker and the sequence holding it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct MarkerEntity {
    /// The sequence holding the marker.
    pub sequence_id: SequenceId,
    /// The marker, its fields flattened next to `sequenceId`.
    #[serde(flatten)]
    pub marker: Marker,
}

/// What one commit changed: entities to put in place (whole, replacing any with the same id) and
/// ids of entities that are gone. Carried by [`crate::CoreEvent::ProjectPatched`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ProjectPatch {
    /// The project the commit belongs to.
    pub project_id: ProjectId,
    /// Revision after the commit. A client whose revision is not `revision − 1` missed a patch
    /// and must reload the snapshot (`getProjectSnapshot`).
    pub revision: u64,
    /// Entities added or changed.
    pub upserts: EntityUpserts,
    /// Entities removed.
    pub deletes: EntityDeletes,
}

/// Entities added or changed by a commit.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct EntityUpserts {
    /// The project's own fields, when they changed.
    pub project: Option<ProjectHeader>,
    /// Sequences added or changed (including their track and marker lists).
    pub sequences: Vec<SequenceEntity>,
    /// Tracks added or changed (including their item lists).
    pub tracks: Vec<TrackEntity>,
    /// Items added or changed.
    pub items: Vec<ItemEntity>,
    /// Markers added or changed.
    pub markers: Vec<MarkerEntity>,
    /// Assets added or changed.
    pub assets: Vec<Asset>,
}

/// Ids of the entities a commit removed.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct EntityDeletes {
    /// Sequences removed.
    pub sequences: Vec<SequenceId>,
    /// Tracks removed.
    pub tracks: Vec<TrackId>,
    /// Items removed.
    pub items: Vec<ItemId>,
    /// Markers removed.
    pub markers: Vec<MarkerId>,
    /// Assets removed.
    pub assets: Vec<se_domain::AssetId>,
}

#[cfg(test)]
mod tests {
    use se_domain::test_support::example_project;

    use super::*;

    #[test]
    fn item_entities_flatten_the_item_next_to_its_track() {
        let project = example_project();
        let track = &project.sequences[0].tracks[0];
        let entity = ItemEntity {
            track_id: track.id,
            item: track.items[0].clone(),
        };
        let json = serde_json::to_value(&entity).unwrap();
        assert_eq!(json["trackId"], track.id.to_string());
        assert_eq!(json["id"], track.items[0].id.to_string());
        assert_eq!(json["kind"]["kind"], "mediaClip");
        assert_eq!(serde_json::from_value::<ItemEntity>(json).unwrap(), entity);
    }

    #[test]
    fn marker_entities_flatten_the_marker_next_to_its_sequence() {
        let project = example_project();
        let sequence = &project.sequences[0];
        let entity = MarkerEntity {
            sequence_id: sequence.id,
            marker: sequence.markers[0].clone(),
        };
        let json = serde_json::to_value(&entity).unwrap();
        assert_eq!(json["sequenceId"], sequence.id.to_string());
        assert_eq!(json["label"], sequence.markers[0].label.as_str());
        assert_eq!(
            serde_json::from_value::<MarkerEntity>(json).unwrap(),
            entity
        );
    }

    #[test]
    fn patches_use_camel_case_lists() {
        let patch = ProjectPatch {
            project_id: example_project().id,
            revision: 7,
            upserts: EntityUpserts::default(),
            deletes: EntityDeletes::default(),
        };
        let json = serde_json::to_value(&patch).unwrap();
        assert_eq!(json["revision"], 7);
        assert_eq!(json["upserts"]["project"], serde_json::Value::Null);
        assert_eq!(json["deletes"]["items"], serde_json::json!([]));
    }
}

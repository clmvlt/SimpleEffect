//! DTOs of the item edit commands (prompt 11, ADR-0021): where items go, how they treat what is
//! already there, which edges move, and where dropped media lands.

use se_domain::{AssetId, Flicks, Item, ItemEdge, ItemId, ProjectId, SequenceId, TrackId};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// How items placed on tracks treat what is already there.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum PlacementMode {
    /// The placed items replace what they cover on their tracks: covered items are removed,
    /// partly covered ones trimmed, items spanning them split around them.
    Overwrite,
    /// Time is opened at the earliest placed start, as long as the placed block, on the target
    /// tracks and the tracks whose linked items must follow: later items move right.
    Insert,
}

/// Where an existing item goes (`moveItems`) or where its copy goes (`duplicateItems`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ItemPlacement {
    /// The item moved or copied.
    pub item_id: ItemId,
    /// The track it (or its copy) ends up on, in the item's sequence.
    pub track_id: TrackId,
    /// Its new start (≥ 0).
    pub start: Flicks,
}

/// An item given by content (`pasteItems`): the clipboard keeps copies because the originals
/// may be gone (cut, then pasted).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PastedItem {
    /// The track receiving the copy.
    pub track_id: TrackId,
    /// The content copied, already at its new start. Its id, revision and origin are replaced
    /// (the copy is a new user item); items sharing a link group stay linked together in a new
    /// group.
    pub item: Item,
}

/// One edge of an item moved (`trimItems`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ItemTrim {
    /// The item trimmed.
    pub item_id: ItemId,
    /// The edge moved.
    pub edge: ItemEdge,
    /// The new position of that edge in the sequence.
    pub to: Flicks,
}

/// The track a part (picture or sound) of dropped media goes to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum TrackTarget {
    /// A track of the sequence (of the part's kind).
    Existing {
        /// The track.
        track_id: TrackId,
    },
    /// A new track of the part's kind, added by the command (on top of the video tracks, below
    /// the audio tracks) and named like `addTrack` names them.
    New,
}

/// Media dropped on a sequence (`placeMedia`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PlaceMediaRequest {
    /// The open project.
    pub project_id: ProjectId,
    /// The sequence receiving the clips.
    pub sequence_id: SequenceId,
    /// The assets, placed one after the other in this order.
    pub asset_ids: Vec<AssetId>,
    /// Start of the first clip (≥ 0).
    pub start: Flicks,
    /// Where pictures go (video assets, images).
    pub video_track: TrackTarget,
    /// Where sound goes (audio assets, the sound of video assets): a video with sound becomes
    /// two linked items.
    pub audio_track: TrackTarget,
    /// What happens to items already there.
    pub mode: PlacementMode,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn track_targets_are_tagged_by_kind() {
        let existing = TrackTarget::Existing {
            track_id: TrackId::from_test_index(4),
        };
        assert_eq!(
            serde_json::to_string(&existing).unwrap(),
            r#"{"kind":"existing","trackId":"00000000-0004-7000-8000-000000000000"}"#
        );
        assert_eq!(
            serde_json::to_string(&TrackTarget::New).unwrap(),
            r#"{"kind":"new"}"#
        );
    }

    #[test]
    fn placements_and_modes_are_camel_case() {
        let placement = ItemPlacement {
            item_id: ItemId::from_test_index(1),
            track_id: TrackId::from_test_index(2),
            start: Flicks::ZERO,
        };
        let json = serde_json::to_string(&placement).unwrap();
        assert!(json.contains(r#""itemId""#) && json.contains(r#""trackId""#));
        assert_eq!(
            serde_json::to_string(&PlacementMode::Insert).unwrap(),
            r#""insert""#
        );
    }
}

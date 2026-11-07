//! DTOs of the sound and transition commands (prompt 14): item patches and new transitions.

use se_domain::{Flicks, ItemId, ItemPatch, TrackId, TransitionAlignment};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A typed change of one item (`updateItems`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ItemUpdate {
    /// The item changed.
    pub item_id: ItemId,
    /// What changes; its kind must match the item's.
    pub patch: ItemPatch,
}

/// A cross-fade wanted on a cut (`addTransitions`): its kind follows the track (dissolve on video
/// tracks, sound cross-fade on audio tracks).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct NewTransition {
    /// The track of the cut.
    pub track_id: TrackId,
    /// The item ending at the cut.
    pub left_item_id: ItemId,
    /// The item starting at the cut.
    pub right_item_id: ItemId,
    /// The wanted length (shortened to what the cut allows).
    pub duration: Flicks,
    /// Where the transition sits on the cut.
    pub alignment: TransitionAlignment,
}

#[cfg(test)]
mod tests {
    use se_domain::test_support::millis;

    use super::*;

    #[test]
    fn new_transitions_use_camel_case_fields() {
        let transition = NewTransition {
            track_id: TrackId::from_test_index(1),
            left_item_id: ItemId::from_test_index(1),
            right_item_id: ItemId::from_test_index(2),
            duration: millis(1000),
            alignment: TransitionAlignment::Center,
        };
        let json = serde_json::to_value(transition).unwrap();
        assert_eq!(json["leftItemId"], "00000000-0001-7000-8000-000000000000");
        assert_eq!(json["alignment"], "center");
        assert_eq!(json["duration"], 705_600_000);
    }
}

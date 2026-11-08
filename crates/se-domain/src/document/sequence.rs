//! [`Sequence`]: a timeline with a format, tracks and markers.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Format, Marker, Track};
use crate::{ids::SequenceId, time::Flicks};

/// A timeline. A project may hold several (e.g. a 16:9 edit and its 9:16 version).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Sequence {
    /// Stable id.
    pub id: SequenceId,
    /// Name shown in the UI.
    pub name: String,
    /// Frame size and rates.
    pub format: Format,
    /// Video and audio tracks; video tracks stack in this order (first = bottom).
    pub tracks: Vec<Track>,
    /// Labelled instants, in any order.
    pub markers: Vec<Marker>,
}

impl Sequence {
    /// Where its content ends: the end of its last item (zero when it is empty). Playback and
    /// agents treat this as the sequence's duration.
    #[must_use]
    pub fn content_end(&self) -> Flicks {
        self.tracks
            .iter()
            .flat_map(|track| &track.items)
            .map(|item| item.timeline_range.end())
            .max()
            .unwrap_or(Flicks::ZERO)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::example_project;

    use super::*;

    #[test]
    fn content_ends_with_the_last_item() {
        let mut sequence = example_project().sequences.remove(0);
        let last = sequence
            .tracks
            .iter()
            .flat_map(|track| &track.items)
            .map(|item| item.timeline_range.end())
            .max()
            .unwrap();
        assert_eq!(sequence.content_end(), last);
        for track in &mut sequence.tracks {
            track.items.clear();
        }
        assert_eq!(sequence.content_end(), Flicks::ZERO);
    }
}

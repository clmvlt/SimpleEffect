//! [`Transition`]: a cross-fade between two adjacent items of a track.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::TrackKind;
use crate::{
    ids::{ItemId, TransitionId},
    time::{Flicks, TimeRange},
};

/// A cross-fade over the cut between two adjacent items of the same track: during its span the
/// outgoing item keeps playing past its end and the incoming one starts before its start, using
/// their **handles** (media beyond their edges, see [`super::item_handles`]).
///
/// The track lists its transitions; the items do not refer to them. Rules (checked by
/// [`crate::validate_project`]): both items are on the track, `left_item` ends where
/// `right_item` starts, the kind matches the track kind, the span fits in both items and their
/// handles, and an item takes part in at most one transition on each side, without overlap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Transition {
    /// Stable id, unique in the project.
    pub id: TransitionId,
    /// Picture dissolve or sound cross-fade.
    pub kind: TransitionKind,
    /// The outgoing item (it ends at the cut).
    pub left_item: ItemId,
    /// The incoming item (it starts at the cut).
    pub right_item: ItemId,
    /// Length of the cross-fade (> 0).
    pub duration: Flicks,
    /// Where the span sits relative to the cut.
    pub alignment: TransitionAlignment,
}

impl Transition {
    /// How much of the span lies before the cut and how much after: half each way (the extra
    /// flick of an odd duration after the cut) when centred, all after when it starts at the
    /// cut, all before when it ends there.
    #[must_use]
    pub const fn extent(&self) -> (Flicks, Flicks) {
        let duration = self.duration.get();
        let (before, after) = match self.alignment {
            TransitionAlignment::Center => (duration / 2, duration - duration / 2),
            TransitionAlignment::Start => (0, duration),
            TransitionAlignment::End => (duration, 0),
        };
        (flicks(before), flicks(after))
    }

    /// The span of the transition around the cut at `cut`; `None` if not representable.
    #[must_use]
    pub fn span(&self, cut: Flicks) -> Option<TimeRange> {
        let (before, _) = self.extent();
        TimeRange::new(cut.checked_sub(before)?, self.duration).ok()
    }
}

/// A part of a valid duration: never fails, `extent` only splits a duration in range.
const fn flicks(value: i64) -> Flicks {
    match Flicks::new(value) {
        Ok(flicks) => flicks,
        Err(_) => Flicks::ZERO,
    }
}

/// What a [`Transition`] blends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum TransitionKind {
    /// The pictures dissolve into each other (video tracks).
    CrossDissolve,
    /// The sounds cross-fade at constant power (audio tracks).
    AudioCrossfade,
}

impl TransitionKind {
    /// The kind of transition a track of `kind` holds.
    #[must_use]
    pub const fn for_track(kind: TrackKind) -> Self {
        match kind {
            TrackKind::Video => Self::CrossDissolve,
            TrackKind::Audio => Self::AudioCrossfade,
        }
    }

    /// The kind of track this transition belongs on.
    #[must_use]
    pub const fn track_kind(self) -> TrackKind {
        match self {
            Self::CrossDissolve => TrackKind::Video,
            Self::AudioCrossfade => TrackKind::Audio,
        }
    }
}

/// Where a [`Transition`]'s span sits relative to its cut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum TransitionAlignment {
    /// Centred on the cut: both items need handles of half the duration.
    #[default]
    Center,
    /// Starts at the cut: only the outgoing item plays past its end.
    Start,
    /// Ends at the cut: only the incoming item starts early.
    End,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn transition(duration: i64, alignment: TransitionAlignment) -> Transition {
        Transition {
            id: TransitionId::from_test_index(1),
            kind: TransitionKind::CrossDissolve,
            left_item: ItemId::from_test_index(1),
            right_item: ItemId::from_test_index(2),
            duration: Flicks::new(duration).unwrap(),
            alignment,
        }
    }

    fn at(value: i64) -> Flicks {
        Flicks::new(value).unwrap()
    }

    #[test]
    fn extents_follow_the_alignment() {
        assert_eq!(
            transition(11, TransitionAlignment::Center).extent(),
            (at(5), at(6))
        );
        assert_eq!(
            transition(10, TransitionAlignment::Start).extent(),
            (at(0), at(10))
        );
        assert_eq!(
            transition(10, TransitionAlignment::End).extent(),
            (at(10), at(0))
        );
    }

    #[test]
    fn the_span_surrounds_the_cut() {
        let span = transition(10, TransitionAlignment::Center).span(at(100));
        assert_eq!(span, Some(TimeRange::new(at(95), at(10)).unwrap()));
        let span = transition(10, TransitionAlignment::Start).span(at(100));
        assert_eq!(span, Some(TimeRange::new(at(100), at(10)).unwrap()));
    }

    #[test]
    fn serializes_with_camel_case_names() {
        let json = serde_json::to_value(transition(10, TransitionAlignment::Start)).unwrap();
        assert_eq!(json["kind"], "crossDissolve");
        assert_eq!(json["alignment"], "start");
        assert_eq!(json["leftItem"], "00000000-0001-7000-8000-000000000000");
    }

    #[test]
    fn kinds_match_track_kinds() {
        for kind in [TrackKind::Video, TrackKind::Audio] {
            assert_eq!(TransitionKind::for_track(kind).track_kind(), kind);
        }
    }
}

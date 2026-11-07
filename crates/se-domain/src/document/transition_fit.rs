//! How transitions fit on a track: the handles of items, what makes a [`Transition`] invalid and
//! the longest transition a cut allows. Shared by validation, the operations that keep
//! transitions in step with their items, and the use cases that create transitions.

use super::{Item, ItemKind, Track, Transition, TransitionAlignment};
use crate::{
    ids::{AssetId, ItemId, TransitionId},
    operation::ItemEdge,
    time::Flicks,
};

/// Duration of an asset, when known (`None`: unknown duration or unknown asset).
pub type AssetDurations<'a> = &'a dyn Fn(AssetId) -> Option<Flicks>;

/// Timeline time an item can play beyond its edges; `None` is unlimited (stills, texts, shapes,
/// media of unknown duration after the end).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Handles {
    /// Before the item's start (the source played before `source_start`).
    pub before: Option<Flicks>,
    /// After the item's end (the source left after the clip's source end).
    pub after: Option<Flicks>,
}

impl Handles {
    const UNLIMITED: Self = Self {
        before: None,
        after: None,
    };
    const NONE: Self = Self {
        before: Some(Flicks::ZERO),
        after: Some(Flicks::ZERO),
    };
}

/// The handles of `item`: for a media clip, the source before its source start and, when the
/// asset's duration is known, after its source end, both divided by the speed; unlimited for
/// items not playing media.
#[must_use]
pub fn item_handles(item: &Item, durations: AssetDurations<'_>) -> Handles {
    let ItemKind::MediaClip(clip) = &item.kind else {
        return Handles::UNLIMITED;
    };
    let Some(source) = clip.source_range(item.timeline_range.duration()) else {
        return Handles::NONE;
    };
    let after = durations(clip.asset_id).map(|duration| {
        let left = duration.checked_sub(source.end()).unwrap_or(Flicks::ZERO);
        timeline_room(left, clip.speed)
    });
    Handles {
        before: Some(timeline_room(clip.source_start, clip.speed)),
        after,
    }
}

/// The longest timeline time whose source length (`time × speed`, rounded like
/// [`crate::MediaClip::source_range`]) fits in `source`.
fn timeline_room(source: Flicks, speed: f64) -> Flicks {
    if source.get() <= 0 {
        return Flicks::ZERO;
    }
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_precision_loss,
        reason = "flicks stay below 2^53, exact in f64; the result is corrected below"
    )]
    let mut room = (source.get() as f64 / speed)
        .floor()
        .min(Flicks::MAX_SAFE as f64) as i64;
    let fits = |time: i64| {
        Flicks::new(time)
            .ok()
            .and_then(|time| time.checked_mul_f64(speed))
            .is_some_and(|needed| needed <= source)
    };
    while room > 0 && !fits(room) {
        room -= 1;
    }
    Flicks::new(room.max(0)).unwrap_or(Flicks::ZERO)
}

/// Why a [`Transition`] does not fit its track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionProblem {
    /// The duration is zero or negative.
    NotPositive,
    /// A picture dissolve on an audio track or a sound cross-fade on a video track.
    WrongTrackKind,
    /// The left and right items are the same item.
    SameItem,
    /// An item of the transition is not on its track.
    MissingItem {
        /// The item.
        item: ItemId,
    },
    /// The left item does not end where the right item starts.
    NotAdjacent {
        /// End of the left item.
        left_end: Flicks,
        /// Start of the right item.
        right_start: Flicks,
    },
    /// The part of the span on one side of the cut is longer than the item there.
    LongerThanItem {
        /// The item.
        item: ItemId,
        /// Part of the span inside it.
        needed: Flicks,
        /// Its duration.
        available: Flicks,
    },
    /// Another transition already uses the same edge of an item.
    EdgeTaken {
        /// The other transition.
        other: TransitionId,
        /// The item whose edge is shared.
        item: ItemId,
    },
    /// The span overlaps the span of another transition inside a shared item.
    Overlap {
        /// The other transition.
        other: TransitionId,
        /// The shared item.
        item: ItemId,
    },
    /// An item has too little media beyond its edge.
    HandleTooShort {
        /// The item.
        item: ItemId,
        /// Its edge played past.
        edge: ItemEdge,
        /// Timeline time it must play past the edge.
        needed: Flicks,
        /// Timeline time its media allows.
        available: Flicks,
    },
}

impl TransitionProblem {
    /// Whether the problem is missing media (the transition is otherwise well placed).
    #[must_use]
    pub const fn is_handle_problem(&self) -> bool {
        matches!(self, Self::HandleTooShort { .. })
    }
}

/// The first reason `transition` does not fit `track`, or `None` when it fits. Edges and spans
/// are compared with `others` only (validation passes the transitions listed before, so a clash
/// is reported once; operations pass all the others).
#[must_use]
pub fn transition_problem<'a>(
    track: &Track,
    transition: &Transition,
    others: impl IntoIterator<Item = &'a Transition>,
    durations: AssetDurations<'_>,
) -> Option<TransitionProblem> {
    let (left, right) = match placed_items(track, transition) {
        Ok(items) => items,
        Err(problem) => return Some(problem),
    };
    let (before, after) = transition.extent();
    let cut = right.timeline_range.start();
    if let Some(problem) = longer_than_item(left, before).or_else(|| longer_than_item(right, after))
    {
        return Some(problem);
    }
    for other in others {
        if let Some(problem) = clash(track, transition, other, cut, (before, after)) {
            return Some(problem);
        }
    }
    let left_room = item_handles(left, durations).after;
    let right_room = item_handles(right, durations).before;
    handle_problem(left.id, ItemEdge::End, after, left_room)
        .or_else(|| handle_problem(right.id, ItemEdge::Start, before, right_room))
}

/// The two items of `transition`, found on `track` and adjacent, or the structural problem.
fn placed_items<'t>(
    track: &'t Track,
    transition: &Transition,
) -> Result<(&'t Item, &'t Item), TransitionProblem> {
    if transition.duration.get() <= 0 {
        return Err(TransitionProblem::NotPositive);
    }
    if transition.kind.track_kind() != track.kind {
        return Err(TransitionProblem::WrongTrackKind);
    }
    if transition.left_item == transition.right_item {
        return Err(TransitionProblem::SameItem);
    }
    let find = |id: ItemId| find_item(track, id).ok_or(TransitionProblem::MissingItem { item: id });
    let (left, right) = (find(transition.left_item)?, find(transition.right_item)?);
    let (left_end, right_start) = (left.timeline_range.end(), right.timeline_range.start());
    if left_end != right_start {
        return Err(TransitionProblem::NotAdjacent {
            left_end,
            right_start,
        });
    }
    Ok((left, right))
}

fn find_item(track: &Track, id: ItemId) -> Option<&Item> {
    track.items.iter().find(|item| item.id == id)
}

fn longer_than_item(item: &Item, needed: Flicks) -> Option<TransitionProblem> {
    let available = item.timeline_range.duration();
    (needed > available).then_some(TransitionProblem::LongerThanItem {
        item: item.id,
        needed,
        available,
    })
}

fn handle_problem(
    item: ItemId,
    edge: ItemEdge,
    needed: Flicks,
    available: Option<Flicks>,
) -> Option<TransitionProblem> {
    let available = available?;
    (needed > available).then_some(TransitionProblem::HandleTooShort {
        item,
        edge,
        needed,
        available,
    })
}

/// A clash between `transition` (cut at `cut`, extent `(before, after)`) and `other`: the same
/// edge of an item, or spans overlapping inside an item they share.
fn clash(
    track: &Track,
    transition: &Transition,
    other: &Transition,
    cut: Flicks,
    (before, after): (Flicks, Flicks),
) -> Option<TransitionProblem> {
    if other.id == transition.id {
        return None;
    }
    if other.left_item == transition.left_item || other.right_item == transition.right_item {
        let item = if other.left_item == transition.left_item {
            transition.left_item
        } else {
            transition.right_item
        };
        return Some(TransitionProblem::EdgeTaken {
            other: other.id,
            item,
        });
    }
    let (other_before, other_after) = other.extent();
    // `other` ends at the start of our left item: its after-part runs into it from its start.
    if other.right_item == transition.left_item
        && let Some(left) = find_item(track, transition.left_item)
        && sum(left.timeline_range.start(), other_after) > diff(cut, before)
    {
        return Some(TransitionProblem::Overlap {
            other: other.id,
            item: left.id,
        });
    }
    // `other` starts at the end of our right item: its before-part runs into it from its end.
    if other.left_item == transition.right_item
        && let Some(right) = find_item(track, transition.right_item)
        && diff(right.timeline_range.end(), other_before) < sum(cut, after)
    {
        return Some(TransitionProblem::Overlap {
            other: other.id,
            item: right.id,
        });
    }
    None
}

fn sum(time: Flicks, offset: Flicks) -> Flicks {
    time.checked_add(offset).unwrap_or(Flicks::MAX)
}

fn diff(time: Flicks, offset: Flicks) -> Flicks {
    time.checked_sub(offset).unwrap_or(Flicks::MIN)
}

/// The longest transition with `alignment` that fits the cut between `left` and `right` on
/// `track`, given the item durations, their handles and the other transitions on these items
/// (`ignoring` one being resized). Zero when the items are not adjacent or the cut already has
/// a transition.
#[must_use]
pub fn transition_room(
    track: &Track,
    (left, right): (ItemId, ItemId),
    alignment: TransitionAlignment,
    ignoring: Option<TransitionId>,
    durations: AssetDurations<'_>,
) -> Flicks {
    let (Some(left_item), Some(right_item)) = (find_item(track, left), find_item(track, right))
    else {
        return Flicks::ZERO;
    };
    if left == right || left_item.timeline_range.end() != right_item.timeline_range.start() {
        return Flicks::ZERO;
    }
    let others = || {
        track
            .transitions
            .iter()
            .filter(move |other| Some(other.id) != ignoring)
    };
    if others().any(|other| other.left_item == left || other.right_item == right) {
        return Flicks::ZERO;
    }
    let taken_in_left = others()
        .filter(|other| other.right_item == left)
        .map(|other| other.extent().1)
        .max()
        .unwrap_or(Flicks::ZERO);
    let taken_in_right = others()
        .filter(|other| other.left_item == right)
        .map(|other| other.extent().0)
        .max()
        .unwrap_or(Flicks::ZERO);
    let max_before = bounded(
        diff(left_item.timeline_range.duration(), taken_in_left),
        item_handles(right_item, durations).before,
    );
    let max_after = bounded(
        diff(right_item.timeline_range.duration(), taken_in_right),
        item_handles(left_item, durations).after,
    );
    let room = match alignment {
        TransitionAlignment::Center => {
            let twice = |time: Flicks| time.checked_mul(2).unwrap_or(Flicks::MAX);
            twice(max_after).min(sum(twice(max_before), flicks_one()))
        }
        TransitionAlignment::Start => max_after,
        TransitionAlignment::End => max_before,
    };
    room.max(Flicks::ZERO)
}

/// `limit` capped by a handle (`None` = unlimited), never negative.
fn bounded(limit: Flicks, handle: Option<Flicks>) -> Flicks {
    handle
        .map_or(limit, |handle| limit.min(handle))
        .max(Flicks::ZERO)
}

const fn flicks_one() -> Flicks {
    match Flicks::new(1) {
        Ok(one) => one,
        Err(_) => Flicks::ZERO,
    }
}

/// `transition` shortened to what its cut allows now (unchanged when it fits), or `None` when
/// its items are no longer adjacent on `track` or leave no room at all.
#[must_use]
pub fn fitted_transition(
    track: &Track,
    transition: &Transition,
    durations: AssetDurations<'_>,
) -> Option<Transition> {
    if transition.kind.track_kind() != track.kind {
        return None;
    }
    let room = transition_room(
        track,
        (transition.left_item, transition.right_item),
        transition.alignment,
        Some(transition.id),
        durations,
    );
    (room.get() > 0).then(|| Transition {
        duration: transition.duration.min(room),
        ..transition.clone()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AssetId, ItemKind, MediaClip, TrackKind, TransitionKind,
        test_support::{item, millis, range_ms, track},
    };

    const ASSET_LENGTH_MS: i64 = 20_000;

    #[allow(
        clippy::unnecessary_wraps,
        reason = "stands for an asset-duration lookup, which may find nothing"
    )]
    fn durations(_: AssetId) -> Option<Flicks> {
        Some(millis(ASSET_LENGTH_MS))
    }

    /// A clip of the 20 s asset on `[start, start + duration)` playing from `source_ms`.
    fn clip(index: u64, start_ms: i64, duration_ms: i64, source_ms: i64) -> Item {
        let clip = MediaClip {
            source_start: millis(source_ms),
            ..MediaClip::new(AssetId::from_test_index(1))
        };
        item(
            index,
            range_ms(start_ms, duration_ms),
            ItemKind::MediaClip(clip),
        )
    }

    fn dissolve(id: u64, left: u64, right: u64, duration_ms: i64) -> Transition {
        Transition {
            id: TransitionId::from_test_index(id),
            kind: TransitionKind::CrossDissolve,
            left_item: ItemId::from_test_index(left),
            right_item: ItemId::from_test_index(right),
            duration: millis(duration_ms),
            alignment: TransitionAlignment::Center,
        }
    }

    /// Clips 1 `[0, 4 s)` from 2 s and 2 `[4 s, 8 s)` from 1 s, then 3 `[8 s, 10 s)` from 5 s.
    fn three_clips() -> Track {
        track(
            1,
            TrackKind::Video,
            "V1",
            vec![
                clip(1, 0, 4000, 2000),
                clip(2, 4000, 4000, 1000),
                clip(3, 8000, 2000, 5000),
            ],
        )
    }

    fn problem(track: &Track, transition: &Transition) -> Option<TransitionProblem> {
        transition_problem(track, transition, &track.transitions, &durations)
    }

    #[test]
    fn handles_come_from_the_media_around_the_source_range() {
        let handles = item_handles(&clip(1, 0, 4000, 2000), &durations);
        assert_eq!(handles.before, Some(millis(2000)));
        assert_eq!(handles.after, Some(millis(14_000)));
        let mut fast = clip(1, 0, 4000, 2000);
        if let ItemKind::MediaClip(clip) = &mut fast.kind {
            clip.speed = 2.0;
        }
        let handles = item_handles(&fast, &durations);
        assert_eq!(handles.before, Some(millis(1000)));
        assert_eq!(handles.after, Some(millis(5000)), "(20 − 10) s / 2");
        assert_eq!(item_handles(&fast, &|_| None).after, None);
    }

    #[test]
    fn a_dissolve_within_the_handles_fits() {
        let track = three_clips();
        assert_eq!(problem(&track, &dissolve(1, 1, 2, 2000)), None);
    }

    #[test]
    fn handles_limit_the_span() {
        let track = three_clips();
        // Clip 2 starts 1 s into its media: a centred 3 s dissolve needs 1.5 s before it.
        assert_eq!(
            problem(&track, &dissolve(1, 1, 2, 3000)),
            Some(TransitionProblem::HandleTooShort {
                item: ItemId::from_test_index(2),
                edge: ItemEdge::Start,
                needed: millis(1500),
                available: millis(1000),
            })
        );
    }

    #[test]
    fn items_must_be_adjacent_on_the_track() {
        let track = three_clips();
        assert_eq!(
            problem(&track, &dissolve(1, 1, 3, 1000)),
            Some(TransitionProblem::NotAdjacent {
                left_end: millis(4000),
                right_start: millis(8000),
            })
        );
        assert_eq!(
            problem(&track, &dissolve(1, 1, 9, 1000)),
            Some(TransitionProblem::MissingItem {
                item: ItemId::from_test_index(9)
            })
        );
        let mut audio = dissolve(1, 1, 2, 1000);
        audio.kind = TransitionKind::AudioCrossfade;
        assert_eq!(
            problem(&track, &audio),
            Some(TransitionProblem::WrongTrackKind)
        );
    }

    #[test]
    fn transitions_sharing_an_item_must_not_overlap() {
        let mut track = three_clips();
        track.transitions = vec![dissolve(1, 1, 2, 2000)];
        // Clip 2 lasts 4 s with 1 s taken at its start: 3.5 s ending at its end overlap it.
        let mut second = dissolve(2, 2, 3, 1000);
        second.alignment = TransitionAlignment::End;
        second.duration = millis(3500);
        assert_eq!(
            problem(&track, &second),
            Some(TransitionProblem::Overlap {
                other: TransitionId::from_test_index(1),
                item: ItemId::from_test_index(2),
            })
        );
        second.duration = millis(3000);
        assert_eq!(problem(&track, &second), None);
        assert_eq!(
            problem(&track, &dissolve(3, 1, 2, 500)),
            Some(TransitionProblem::EdgeTaken {
                other: TransitionId::from_test_index(1),
                item: ItemId::from_test_index(1),
            })
        );
    }

    #[test]
    fn the_room_is_the_longest_fitting_transition() {
        let mut track = three_clips();
        let pair = (ItemId::from_test_index(1), ItemId::from_test_index(2));
        // Before the cut: clip 2's 1 s handle; after it: clip 1's 14 s handle, clip 2's 4 s.
        let centred = transition_room(&track, pair, TransitionAlignment::Center, None, &durations);
        assert_eq!(centred, millis(2000).checked_add(flicks_one()).unwrap());
        let start = transition_room(&track, pair, TransitionAlignment::Start, None, &durations);
        assert_eq!(start, millis(4000));
        let end = transition_room(&track, pair, TransitionAlignment::End, None, &durations);
        assert_eq!(end, millis(1000));
        let mut fitting = dissolve(1, 1, 2, 1000);
        fitting.duration = centred;
        assert_eq!(problem(&track, &fitting), None);
        track.transitions = vec![dissolve(1, 1, 2, 1000)];
        let taken = transition_room(&track, pair, TransitionAlignment::Start, None, &durations);
        assert_eq!(taken, Flicks::ZERO);
        let resized = transition_room(
            &track,
            pair,
            TransitionAlignment::Start,
            Some(TransitionId::from_test_index(1)),
            &durations,
        );
        assert_eq!(resized, millis(4000));
    }

    #[test]
    fn fitting_shortens_or_drops() {
        let track = three_clips();
        let shortened = fitted_transition(&track, &dissolve(1, 1, 2, 9000), &durations).unwrap();
        assert_eq!(
            shortened.duration,
            millis(2000).checked_add(flicks_one()).unwrap()
        );
        assert_eq!(
            fitted_transition(&track, &dissolve(1, 1, 2, 1000), &durations),
            Some(dissolve(1, 1, 2, 1000))
        );
        assert_eq!(
            fitted_transition(&track, &dissolve(1, 1, 3, 1000), &durations),
            None
        );
    }
}

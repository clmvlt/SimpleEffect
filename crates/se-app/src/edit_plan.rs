//! [`EditPlan`]: the domain operations of one user gesture, planned against a draft of the
//! document (ADR-0021).
//!
//! A timeline gesture (moving clips over others, a ripple delete, an insert drop…) is several
//! domain operations whose later steps depend on the earlier ones: what lies under a moved clip
//! is only known once the clip is lifted. The plan applies each operation to a draft with
//! [`se_domain::apply`] as it is added, so every step is checked by the domain and planned
//! against the state it will meet. The project's actor then commits the operations as one
//! transaction: one undo step.
//!
//! - `lookup`: finding items, tracks and sequences, and checking that one edit stays in one
//!   sequence.
//! - `placement`: lifting items and placing items with a [`PlacementMode`].
//! - `overwrite`: emptying a range of a track (overwrite placement).
//! - `ripple`: the tracks a ripple edit shifts, opening and removing time, ripple deletes.
//! - `new_items`: items of dropped media and copies of items.
//!
//! Item edits drop the transitions they break (a domain rule); [`EditPlan::finish`] puts back
//! those whose two items touch again once the whole gesture is planned (both items of a
//! transition moved together, a ripple trim at its cut), shortened to fit.
//!
//! [`PlacementMode`]: se_protocol::PlacementMode

mod lookup;
mod new_items;
mod overwrite;
mod placement;
mod ripple;

pub(crate) use lookup::{distinct, find_item, find_sequence, find_track, sequence_of};
pub(crate) use new_items::{copies_of, media_parts, starting_at};
pub(crate) use placement::{lift, place};
pub(crate) use ripple::{concerned_tracks, open_time, remove_time, ripple_delete};

pub use new_items::DEFAULT_IMAGE_DURATION;

use se_domain::{ItemId, Operation, Project, TransitionSnapshot};

use crate::{AppError, new_ids::NewIds};

/// Operations of one gesture and the draft they lead to.
///
/// Operations are queued by [`EditPlan::push`] and applied to the draft only when the draft is
/// read ([`EditPlan::project`]), all pending ones at once with [`se_domain::apply_all`]: the
/// document is validated once per read rather than once per operation, which matters when a
/// gesture moves hundreds of items. Refusals of queued operations surface at the next read, or
/// when the actor commits the plan.
pub(crate) struct EditPlan<'a> {
    draft: Project,
    ops: Vec<Operation>,
    /// How many of `ops` the draft already reflects.
    applied: usize,
    ids: NewIds<'a>,
    /// The transitions before the gesture, to put back those it removed and that fit again.
    transitions: TransitionSnapshot,
}

impl<'a> EditPlan<'a> {
    /// An empty plan over a copy of `project`; new ids come from `ids`.
    pub(crate) fn new(project: &Project, ids: NewIds<'a>) -> Self {
        Self {
            draft: project.clone(),
            ops: Vec::new(),
            applied: 0,
            ids,
            transitions: TransitionSnapshot::of(project),
        }
    }

    /// The document as the operations planned so far leave it.
    ///
    /// # Errors
    /// The domain refuses a queued operation (overlap, locked track…).
    pub(crate) fn project(&mut self) -> Result<&Project, AppError> {
        if self.applied < self.ops.len() {
            let applied = se_domain::apply_all(&self.draft, &self.ops[self.applied..])?;
            self.draft = applied.project;
            self.applied = self.ops.len();
        }
        Ok(&self.draft)
    }

    /// Where new ids come from.
    pub(crate) const fn ids(&self) -> NewIds<'a> {
        self.ids
    }

    /// Queues `operation` (applied to the draft at the next read).
    pub(crate) fn push(&mut self, operation: impl Into<Operation>) {
        self.ops.push(operation.into());
    }

    /// The planned operations, in order, followed by the restoration of the transitions the
    /// gesture removed and whose items touch again (one at a time: each may change the room left
    /// for the next).
    ///
    /// # Errors
    /// The domain refuses a queued operation.
    pub(crate) fn finish(mut self) -> Result<Vec<Operation>, AppError> {
        let renamed = self.cut_parts();
        let before = std::mem::take(&mut self.transitions);
        for _ in 0..before.len() {
            let draft = self.project()?;
            let Some(addition) = before.restorations(draft, &renamed).into_iter().next() else {
                break;
            };
            self.push(addition);
        }
        Ok(self.ops)
    }

    /// Each item cut by a planned operation, with the new item holding its end.
    fn cut_parts(&self) -> Vec<(ItemId, ItemId)> {
        let mut parts = Vec::new();
        for operation in &self.ops {
            match operation {
                Operation::SplitItem(split) => parts.push((split.item, split.new_item_id)),
                Operation::RippleDeleteRange(ripple) => parts.extend(ripple.split_ids.clone()),
                Operation::RippleDeleteRanges(ripple) => parts.extend(
                    ripple
                        .split_ids
                        .iter()
                        .filter_map(|(item, ids)| ids.last().map(|last| (*item, *last))),
                ),
                Operation::InsertGap(gap) => parts.extend(gap.split_ids.clone()),
                _ => {}
            }
        }
        parts
    }
}

#[cfg(test)]
mod testing {
    //! Helpers of the planner's unit tests: a sequence V1, V2, A1 with items at whole seconds.

    use se_domain::{
        Flicks, Item, ItemId, ItemKind, LinkGroupId, MediaClip, Project, TimeRange, TrackId,
        TrackKind,
        test_support::{example_ids, example_project, item, track},
    };

    pub(super) const V1: u64 = 101;
    pub(super) const V2: u64 = 102;
    pub(super) const A1: u64 = 103;

    /// `seconds` as flicks.
    pub(super) fn secs(seconds: i64) -> Flicks {
        Flicks::new(seconds * Flicks::PER_SECOND).unwrap()
    }

    /// An item of the example's video asset over `[start, end)` seconds, linked to `group`.
    pub(super) fn clip(index: u64, start: i64, end: i64, group: Option<u64>) -> Item {
        let range = TimeRange::from_start_end(secs(start), secs(end)).unwrap();
        let mut item = item(
            index,
            range,
            ItemKind::MediaClip(MediaClip::new(example_ids::VIDEO_ASSET)),
        );
        item.link_group = group.map(LinkGroupId::from_test_index);
        item
    }

    /// The example project whose first sequence holds exactly these tracks.
    pub(super) fn project_with(video1: Vec<Item>, video2: Vec<Item>, audio1: Vec<Item>) -> Project {
        let mut project = example_project();
        let sequence = &mut project.sequences[0];
        sequence.tracks = vec![
            track(V1, TrackKind::Video, "V1", video1),
            track(V2, TrackKind::Video, "V2", video2),
            track(A1, TrackKind::Audio, "A1", audio1),
        ];
        project.sequences.truncate(1);
        project
    }

    /// The track id of a test index.
    pub(super) fn track_id(index: u64) -> TrackId {
        TrackId::from_test_index(index)
    }

    /// The item id of a test index.
    pub(super) fn item_id(index: u64) -> ItemId {
        ItemId::from_test_index(index)
    }

    /// `(start, end)` in seconds of the items of a track.
    pub(super) fn spans(project: &Project, track: u64) -> Vec<(i64, i64)> {
        let track = project.sequences[0]
            .tracks
            .iter()
            .find(|candidate| candidate.id == track_id(track))
            .unwrap();
        track
            .items
            .iter()
            .map(|item| {
                let range = item.timeline_range;
                (
                    range.start().get() / Flicks::PER_SECOND,
                    range.end().get() / Flicks::PER_SECOND,
                )
            })
            .collect()
    }
}

//! What agents ask for ([`AgentEdit`]) and what they get back ([`AgentEditReport`],
//! [`TaskChanges`]). Times are flicks and entities ids: se-mcp converts the agents' seconds,
//! timecodes and names at the boundary.

use std::sync::Arc;

use se_domain::{
    AnimationPhase, ChangesetId, Flicks, ItemEdge, ItemId, ItemKind, ItemPatch, MarkerColor,
    Project, PropertyAnimation, TimeRange, TrackId, TransitionAlignment,
};

/// An item an edit works on: an existing one, or one created by an earlier edit of the same
/// call (`apply_batch` references).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemRef {
    /// An item of the project.
    Id(ItemId),
    /// The item created by edit number `edit` (0-based) of the same call.
    Created {
        /// Index of that edit.
        edit: usize,
    },
}

impl From<ItemId> for ItemRef {
    fn from(id: ItemId) -> Self {
        Self::Id(id)
    }
}

/// An animation preset to apply to an item created by [`AgentEdit::AddItem`].
#[derive(Debug, Clone, PartialEq)]
pub struct PresetSpec {
    /// The preset (`fade`, `pop`…; see `list_animation_presets`).
    pub preset: String,
    /// Entrance, exit or emphasis.
    pub phase: AnimationPhase,
    /// Length of the animation; the preset's default when `None`.
    pub duration: Option<Flicks>,
}

/// A new text, shape or image placed by an agent.
#[derive(Debug, Clone, PartialEq)]
pub struct NewAgentItem {
    /// What it shows (a `Text`, `Shape` or `Image` kind, fully styled).
    pub kind: ItemKind,
    /// Where it sits in the task's sequence.
    pub range: TimeRange,
    /// The video track; `None` picks the highest free track of the task (or a new one on top
    /// when the task may use every track).
    pub track: Option<TrackId>,
    /// Animations applied to it in the same edit.
    pub presets: Vec<PresetSpec>,
}

/// One edit asked by an agent. Several edits of one call form one transaction: all or nothing.
#[derive(Debug, Clone, PartialEq)]
pub enum AgentEdit {
    /// Adds a text, shape or image.
    AddItem(Box<NewAgentItem>),
    /// Changes static content of an item (text, style, clip speed, fades, reframing…).
    UpdateItem {
        /// The item.
        item: ItemRef,
        /// The change, typed by item kind.
        patch: ItemPatch,
        /// The revision the agent read (`REVISION_CONFLICT` when the item changed since).
        expected_revision: Option<u64>,
    },
    /// Sets one animatable property: a static value or keyframes (volume automation among them).
    SetProperty {
        /// The item.
        item: ItemRef,
        /// The property and its value.
        animation: PropertyAnimation,
        /// The revision the agent read.
        expected_revision: Option<u64>,
    },
    /// Applies an animation preset.
    ApplyPreset {
        /// The item.
        item: ItemRef,
        /// The preset and its phase and length.
        preset: PresetSpec,
        /// Where the animation starts, relative to the item's start; `None` for the phase's
        /// natural place (an entrance at the start, an exit ending at the end).
        start: Option<Flicks>,
        /// The revision the agent read.
        expected_revision: Option<u64>,
    },
    /// Deletes items (and their linked partners), leaving gaps.
    DeleteItems {
        /// The items, each with the revision the agent read.
        items: Vec<(ItemRef, Option<u64>)>,
    },
    /// Moves an item (its linked partners move by the same amount, on their own tracks).
    MoveItem {
        /// The item.
        item: ItemRef,
        /// Its new track; `None` keeps it.
        track: Option<TrackId>,
        /// Its new start.
        start: Flicks,
        /// The revision the agent read.
        expected_revision: Option<u64>,
    },
    /// Moves one edge of an item (and the same edge of linked partners sitting at the same
    /// instant).
    TrimItem {
        /// The item.
        item: ItemRef,
        /// Which edge.
        edge: ItemEdge,
        /// Where it goes, in sequence time.
        to: Flicks,
        /// The revision the agent read.
        expected_revision: Option<u64>,
    },
    /// Cuts items in two at an instant (and their linked partners crossing it).
    Split {
        /// The items; `None` cuts every item crossing `at` on the task's unlocked tracks.
        items: Option<Vec<ItemRef>>,
        /// The instant, in the task's sequence.
        at: Flicks,
    },
    /// Removes a time range from tracks and closes it (what follows moves left).
    RippleDelete {
        /// The range, in the task's sequence.
        range: TimeRange,
        /// The tracks; `None` for the task's tracks (every unlocked track when unrestricted).
        tracks: Option<Vec<TrackId>>,
    },
    /// Opens empty time on tracks (what follows moves right).
    InsertGap {
        /// Where.
        at: Flicks,
        /// How long.
        duration: Flicks,
        /// The tracks; `None` for the task's tracks (every unlocked track when unrestricted).
        tracks: Option<Vec<TrackId>>,
    },
    /// Adds a cross-fade on the cut between two adjacent items of a track.
    AddTransition {
        /// The item ending at the cut.
        left: ItemRef,
        /// The item starting at the cut.
        right: ItemRef,
        /// The wanted length (shortened to what the cut allows).
        duration: Flicks,
        /// Where it sits on the cut.
        alignment: TransitionAlignment,
    },
    /// Adds a marker to the task's sequence.
    AddMarker {
        /// Where.
        time: Flicks,
        /// Its label.
        label: String,
        /// Its colour.
        color: MarkerColor,
    },
}

/// What one call of [`super::AgentEditService::apply`] did.
#[derive(Debug, Clone)]
pub struct AgentEditReport {
    /// The project's revision after the call.
    pub revision: u64,
    /// For each edit, in order, the items it created.
    pub created: Vec<Vec<ItemId>>,
    /// Items added or changed that still exist, in id order (their new revisions are in
    /// `project`).
    pub changed: Vec<ItemId>,
    /// Items removed.
    pub removed: Vec<ItemId>,
    /// The document right after the call.
    pub project: Arc<Project>,
}

/// What a task changed so far: its open changeset, summarised.
#[derive(Debug, Clone)]
pub struct TaskChanges {
    /// The open changeset (`None` while the task changed nothing, or after a revert).
    pub changeset_id: Option<ChangesetId>,
    /// How many edits (transactions) it holds.
    pub edits: usize,
    /// Items the task created that still exist.
    pub created: Vec<ItemId>,
    /// Items that existed before the task, changed by it, that still exist.
    pub changed: Vec<ItemId>,
    /// Items the task removed (or created then removed).
    pub removed: Vec<ItemId>,
    /// The current document.
    pub project: Arc<Project>,
}

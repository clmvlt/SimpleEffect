//! What agents ask for ([`AgentEdit`]) and what they get back ([`AgentEditReport`],
//! [`TaskChanges`]). Times are flicks and entities ids: se-mcp converts the agents' seconds,
//! timecodes and names at the boundary.

use std::{collections::BTreeMap, sync::Arc};

use se_domain::{
    AnimationPhase, AssetId, CaptionStylePatch, ChangesetId, Flicks, Format, ItemEdge, ItemId,
    ItemKind, ItemPatch, MarkerColor, Project, PropertyAnimation, TemplateDefinition, TemplateId,
    TemplateParamValue, TimeRange, TrackId, TrackingSettings, TransitionAlignment,
    captions::TimedWord,
};

use super::templates::TemplateChange;
use crate::reframing::AssetAnalyses;
use se_protocol::{CaptionSegmentation, CaptionStyleChoice, LibraryTemplate};

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

/// Captions an agent makes from spoken words (prompt 23).
#[derive(Debug, Clone, PartialEq)]
pub struct NewAgentCaptions {
    /// The range to fill (captions of the caption track crossing it are replaced).
    pub range: TimeRange,
    /// The words spoken in the sequence (sequence time), read by the tool beforehand.
    pub words: Vec<TimedWord>,
    /// The style (`None`: the replaced captions' style, else the default preset).
    pub style: Option<CaptionStyleChoice>,
    /// Segmentation overrides.
    pub segmentation: Option<CaptionSegmentation>,
}

/// A new text, shape or image placed by an agent.
#[derive(Debug, Clone, PartialEq)]
pub struct NewAgentItem {
    /// What it shows (a `Text`, `Shape`, `Image` or `MotionTemplate` kind, fully styled).
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
    /// Makes captions from spoken words on the caption track (created on top when missing).
    AddCaptions(Box<NewAgentCaptions>),
    /// Changes the text of a caption (unchanged words keep their timing).
    EditCaptionText {
        /// The caption.
        item: ItemRef,
        /// Its new text.
        text: String,
    },
    /// Puts words of a caption forward or back.
    SetCaptionEmphasis {
        /// The caption.
        item: ItemRef,
        /// Indices of its words.
        words: Vec<u32>,
        /// Forward or back.
        emphasis: bool,
    },
    /// Changes the look of captions (copy on write of styles shared with other captions).
    UpdateCaptionStyle {
        /// The captions.
        items: Vec<ItemRef>,
        /// What changes.
        patch: CaptionStylePatch,
    },
    /// Gives captions a new style made from a preset.
    ApplyCaptionPreset {
        /// The captions.
        items: Vec<ItemRef>,
        /// The preset's id.
        preset: String,
    },
    /// Adds a motion template made of a module the renderer checked (prompt 24).
    CreateTemplate {
        /// What the module declares.
        definition: Box<TemplateDefinition>,
        /// The module.
        source: String,
    },
    /// A new version of a motion template: in place when every instance lies in the task's
    /// scope, else a copy for the instances of the scope (copy on write).
    UpdateTemplate {
        /// The template.
        template: TemplateId,
        /// What the new module declares.
        definition: Box<TemplateDefinition>,
        /// The new module.
        source: String,
    },
    /// Copies a motion template of the app's library into the project (reusing a copy of the
    /// same version) and places an instance (prompt 25); brand-bound parameters start with the
    /// project's brand kit values, `params` win over them.
    InsertLibraryTemplate(Box<LibraryInstance>),
    /// Makes clips of the task's scope follow their subject (prompt 26, ADR-0037), from their
    /// media's analyses.
    ReframeClips(Box<ReframeClips>),
    /// Adds a version of the task's sequence in another format, after it, whose clips follow
    /// their subject (prompt 26). Everything in it is new content, so the scope allows it;
    /// reverting the task removes it.
    AddReframedVersion(Box<ReframedVersion>),
}

/// Clips to reframe automatically (`auto_reframe` on items).
#[derive(Debug, Clone, PartialEq)]
pub struct ReframeClips {
    /// The clips (media clips on video tracks).
    pub items: Vec<ItemRef>,
    /// How to follow the subject.
    pub settings: TrackingSettings,
    /// The faces and shot changes of their media (every one must be there).
    pub analyses: BTreeMap<AssetId, AssetAnalyses>,
}

/// A reframed version of the task's sequence (`auto_reframe` with `target_format`).
#[derive(Debug, Clone, PartialEq)]
pub struct ReframedVersion {
    /// Its name.
    pub name: String,
    /// Its format (the frame rate of the task's sequence).
    pub format: Format,
    /// How to follow the subject.
    pub settings: TrackingSettings,
    /// The faces and shot changes of the media of its clips.
    pub analyses: BTreeMap<AssetId, AssetAnalyses>,
}

/// An instance of a library template asked by an agent (`insert_library_template`).
#[derive(Debug, Clone, PartialEq)]
pub struct LibraryInstance {
    /// The library entry (a motion template).
    pub entry: LibraryTemplate,
    /// Where it sits in the task's sequence.
    pub range: TimeRange,
    /// The video track; `None` picks the highest free track of the task.
    pub track: Option<TrackId>,
    /// Values given by the agent.
    pub params: BTreeMap<String, TemplateParamValue>,
    /// Animations applied to it in the same edit.
    pub presets: Vec<PresetSpec>,
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
    /// Templates created or given a new version, in edit order.
    pub templates: Vec<TemplateChange>,
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

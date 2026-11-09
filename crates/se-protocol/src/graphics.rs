//! DTOs of the graphics and keyframe commands (prompt 15): new texts, shapes and images, batched
//! keyframe changes and the animation preset library.

use se_domain::{
    AnimatableProperty, AnimationPhase, AssetId, Flicks, ItemId, ProjectId, PropertyAnimation,
    SequenceId, ShapeKind, TemplateId, TrackId,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What a new graphic item shows (`addGraphicItem`). It starts with the default look (white
/// Inter text, white shape, image fitted into the frame), centred.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum NewGraphic {
    /// A text.
    Text {
        /// Its content (lines separated by `\n`).
        content: String,
    },
    /// A shape.
    Shape {
        /// Which shape.
        shape: ShapeKind,
    },
    /// An image asset of the project.
    Image {
        /// The image asset.
        asset_id: AssetId,
    },
    /// An instance of a motion template of the project, with its default values (prompt 24).
    Template {
        /// The template.
        template_id: TemplateId,
    },
}

/// A graphic item to add at a time (`addGraphicItem`): it goes on the highest unlocked video
/// track free over its span with nothing above it at that time, or on a new video track on top.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AddGraphicRequest {
    /// The open project.
    pub project_id: ProjectId,
    /// The sequence receiving it.
    pub sequence_id: SequenceId,
    /// Where it starts.
    pub at: Flicks,
    /// How long it lasts (> 0).
    pub duration: Flicks,
    /// What it shows.
    pub graphic: NewGraphic,
}

/// A new value of one animatable property of one item (`setItemAnimations`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct ItemAnimation {
    /// The item animated.
    pub item_id: ItemId,
    /// The property and its new keyframes or static value.
    pub animation: PropertyAnimation,
}

/// An animation preset of the library (`listAnimationPresets`), with what it does to a
/// reference text so clients can preview it without expanding it themselves.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AnimationPresetInfo {
    /// Stable id, given to `applyAnimationPreset`.
    pub id: String,
    /// English name.
    pub name: String,
    /// What it does.
    pub description: String,
    /// Phases it can be applied as (`in` and `out`, or `emphasis`).
    pub phases: Vec<AnimationPhase>,
    /// Suggested duration.
    pub default_duration: Flicks,
    /// Properties it animates (an item must have them all).
    pub properties: Vec<AnimatableProperty>,
    /// Its expansion on a centred « Aa » text lasting one second, per phase.
    pub samples: Vec<AnimationPresetSample>,
}

/// What a preset does in one phase to the reference text of [`AnimationPresetInfo::samples`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct AnimationPresetSample {
    /// The phase.
    pub phase: AnimationPhase,
    /// The animated properties (keyframes over one second).
    pub animations: Vec<PropertyAnimation>,
}

/// Where `addGraphicItem` put the new item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct PlacedGraphic {
    /// The new item.
    pub item_id: ItemId,
    /// Its track (possibly a new one).
    pub track_id: TrackId,
}

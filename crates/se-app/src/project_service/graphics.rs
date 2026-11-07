//! Graphics and keyframes (prompt 15): new texts, shapes and images placed at a time, batched
//! keyframe changes and animation presets. Each command is one user transaction.

use se_domain::{
    AddItem, AddTrack, AnimatableProperty, AnimationPhase, ApplyAnimationPreset, AssetKind, Flicks,
    ImageItem, Item, ItemId, ItemKind, Operation, Origin, ProjectId, Sequence, SetKeyframes,
    ShapeItem, TextItem, TimeRange, TrackId, TrackKind, animation_presets, preset_samples,
};
use se_protocol::{
    AddGraphicRequest, AnimationPresetInfo, AnimationPresetSample, ItemAnimation, NewGraphic,
    PlacedGraphic,
};

use super::{ProjectService, items::labels, tracks::next_track_name};
use crate::{
    AppError,
    edit_plan::{distinct, find_sequence},
    new_project::empty_track,
};

impl ProjectService {
    /// Adds a text, a shape or an image (default look, centred) over `[at, at + duration)` on the
    /// highest free video track, or on a new video track on top (see [`graphic_track`]).
    ///
    /// # Errors
    /// Not open, unknown sequence or asset, an asset that is not an image, a duration that is not
    /// positive, storage errors.
    pub async fn add_graphic_item(
        &self,
        request: &AddGraphicRequest,
    ) -> Result<PlacedGraphic, AppError> {
        let handle = self.handle(request.project_id).await?;
        let view = handle.view().await?;
        let range = TimeRange::new(request.at, request.duration)
            .ok()
            .filter(|range| range.duration().get() > 0)
            .ok_or_else(|| AppError::InvalidEdit {
                reason: "a graphic needs a positive duration that ends within the supported \
                         range"
                    .to_owned(),
            })?;
        let kind = graphic_kind(&view.project, request)?;
        let sequence = find_sequence(&view.project, request.sequence_id)?;
        let ids = self.new_ids();
        let mut ops: Vec<Operation> = Vec::new();
        let track_id = if let Some(track) = graphic_track(sequence, range) {
            track
        } else {
            let name = next_track_name(sequence, TrackKind::Video);
            let track = empty_track(ids, &name, TrackKind::Video)?;
            let track_id = track.id;
            ops.push(
                AddTrack {
                    sequence: sequence.id,
                    index: Some(top_video_index(sequence)),
                    track,
                }
                .into(),
            );
            track_id
        };
        let item = Item {
            id: ids.item()?,
            revision: 0,
            origin: Origin::User,
            timeline_range: range,
            link_group: None,
            kind,
        };
        let item_id = item.id;
        ops.push(
            AddItem {
                track: track_id,
                item,
            }
            .into(),
        );
        let label = match request.graphic {
            NewGraphic::Text { .. } => labels::ADD_TEXT,
            NewGraphic::Shape { .. } => labels::ADD_SHAPE,
            NewGraphic::Image { .. } => labels::ADD_IMAGE,
        };
        self.commit_user_edit(&handle, label, ops).await?;
        Ok(PlacedGraphic { item_id, track_id })
    }

    /// Sets animatable properties of items in one transaction (several properties of one item,
    /// or one property of several items).
    ///
    /// # Errors
    /// Not open, nothing listed, unknown items, properties an item does not have, keyframes the
    /// document refuses, storage errors.
    pub async fn set_item_animations(
        &self,
        project_id: ProjectId,
        animations: &[ItemAnimation],
    ) -> Result<(), AppError> {
        if animations.is_empty() {
            return Err(AppError::InvalidEdit {
                reason: "nothing to animate: list at least one property".to_owned(),
            });
        }
        let handle = self.handle(project_id).await?;
        let ops = animations
            .iter()
            .map(|change| {
                SetKeyframes {
                    item: change.item_id,
                    animation: change.animation.clone(),
                    expected_revision: None,
                }
                .into()
            })
            .collect();
        self.commit_user_edit(&handle, animation_label(animations), ops)
            .await
    }

    /// Applies an animation preset to items (one transaction for all of them).
    ///
    /// # Errors
    /// Not open, nothing listed or an item twice, unknown items or preset, a phase the preset
    /// does not have, an item lacking a property of the preset, a duration that is not positive,
    /// storage errors.
    pub async fn apply_animation_preset(
        &self,
        project_id: ProjectId,
        item_ids: &[ItemId],
        preset: &str,
        phase: AnimationPhase,
        duration: Flicks,
        start: Option<Flicks>,
    ) -> Result<(), AppError> {
        distinct(item_ids.iter().copied())?;
        let handle = self.handle(project_id).await?;
        let ops = item_ids
            .iter()
            .map(|&item| {
                ApplyAnimationPreset {
                    item,
                    preset: preset.to_owned(),
                    phase,
                    duration,
                    start,
                    expected_revision: None,
                }
                .into()
            })
            .collect();
        self.commit_user_edit(&handle, labels::APPLY_ANIMATION, ops)
            .await
    }
}

/// The animation preset library with a preview of each preset on a reference text.
///
/// # Errors
/// [`AppError::Domain`] if the embedded library is broken (a bug).
pub fn animation_preset_infos() -> Result<Vec<AnimationPresetInfo>, AppError> {
    let library = animation_presets()?;
    Ok(library
        .presets
        .iter()
        .map(|preset| AnimationPresetInfo {
            id: preset.id.clone(),
            name: preset.name.clone(),
            description: preset.description.clone(),
            phases: preset.phases.clone(),
            default_duration: preset.default_duration(),
            properties: preset.properties(),
            samples: preset_samples(preset)
                .into_iter()
                .map(|(phase, animations)| AnimationPresetSample { phase, animations })
                .collect(),
        })
        .collect())
}

/// The content of a new graphic item.
fn graphic_kind(
    project: &se_domain::Project,
    request: &AddGraphicRequest,
) -> Result<ItemKind, AppError> {
    Ok(match &request.graphic {
        NewGraphic::Text { content } => ItemKind::Text(TextItem::new(content.clone())),
        NewGraphic::Shape { shape } => ItemKind::Shape(ShapeItem::new(*shape)),
        NewGraphic::Image { asset_id } => {
            let asset = project
                .assets
                .iter()
                .find(|asset| asset.id == *asset_id)
                .ok_or(AppError::AssetNotFound {
                    project_id: request.project_id,
                    asset_id: *asset_id,
                })?;
            if asset.kind != AssetKind::Image {
                return Err(AppError::InvalidEdit {
                    reason: format!(
                        "asset {asset_id} is not an image: place videos and sounds with placeMedia"
                    ),
                });
            }
            ItemKind::Image(ImageItem::new(*asset_id))
        }
    })
}

/// The track a new graphic over `range` goes on: going down from the top video track, the first
/// unlocked, visible track that is free over `range`, as long as no track above it holds
/// something then (so the graphic shows over the picture). `None` when there is none: the
/// graphic gets a new video track on top.
#[must_use]
pub fn graphic_track(sequence: &Sequence, range: TimeRange) -> Option<TrackId> {
    for track in sequence
        .tracks
        .iter()
        .rev()
        .filter(|track| track.kind == TrackKind::Video)
    {
        let busy = track
            .items
            .iter()
            .any(|item| item.timeline_range.intersects(range));
        if busy {
            return None;
        }
        if !track.locked && !track.hidden {
            return Some(track.id);
        }
    }
    None
}

/// The index of a new top video track: right after the last video track (video tracks are listed
/// bottom first).
fn top_video_index(sequence: &Sequence) -> usize {
    sequence
        .tracks
        .iter()
        .rposition(|track| track.kind == TrackKind::Video)
        .map_or(0, |index| index + 1)
}

/// « Transform items » for placement properties, « Change volume » for volumes, « Reframe
/// items » for reframe focuses, « Change keyframes » when keyframes are written, « Edit items »
/// for other static values.
pub(super) fn animation_label(animations: &[ItemAnimation]) -> &'static str {
    use AnimatableProperty as P;
    let properties = || animations.iter().map(|change| change.animation.property());
    if properties()
        .all(|property| matches!(property, P::Position | P::Scale | P::Rotation | P::Anchor))
    {
        labels::TRANSFORM
    } else if properties().all(|property| property == P::VolumeDb) {
        labels::CHANGE_VOLUME
    } else if properties().all(|property| property == P::ReframeFocus) {
        labels::REFRAME
    } else if animations
        .iter()
        .any(|change| change.animation.is_keyframed())
    {
        labels::CHANGE_KEYFRAMES
    } else {
        labels::EDIT_ITEMS
    }
}

#[cfg(test)]
mod tests {
    use se_domain::{
        Animatable, PropertyAnimation, Vec2,
        test_support::{example_project, range_ms},
    };

    use super::*;

    #[test]
    fn graphics_go_above_what_plays_at_that_time() {
        // V1 holds clips over [0 s, 14 s), V2 a text over [1 s, 4 s) and an image over [5 s,
        // 7 s), V3 a template, a shape and a caption over [2 s, 10 s).
        let project = example_project();
        let sequence = &project.sequences[0];
        let v3 = sequence.tracks[2].id;
        assert_eq!(graphic_track(sequence, range_ms(11_000, 2000)), Some(v3));
        assert_eq!(graphic_track(sequence, range_ms(3000, 1000)), None);
        let mut hidden = sequence.clone();
        hidden.tracks[2].hidden = true;
        assert_eq!(
            graphic_track(&hidden, range_ms(11_000, 2000)),
            Some(hidden.tracks[1].id),
            "a hidden track is skipped"
        );
        assert_eq!(top_video_index(sequence), 3);
    }

    #[test]
    fn animation_labels_say_what_changed() {
        let change = |animation| ItemAnimation {
            item_id: ItemId::from_test_index(1),
            animation,
        };
        let moved = change(PropertyAnimation::Position {
            value: Animatable::constant(Vec2::splat(0.2)),
        });
        assert_eq!(
            animation_label(std::slice::from_ref(&moved)),
            labels::TRANSFORM
        );
        let faded = change(PropertyAnimation::Opacity {
            value: Animatable::constant(0.5),
        });
        assert_eq!(animation_label(&[moved, faded]), labels::EDIT_ITEMS);
    }

    #[test]
    fn every_preset_is_listed_with_its_samples() {
        let infos = animation_preset_infos().unwrap();
        let fade = infos.iter().find(|info| info.id == "fade").unwrap();
        assert_eq!(fade.samples.len(), 2);
        assert_eq!(fade.properties, [AnimatableProperty::Opacity]);
        assert!(infos.len() >= 11);
    }
}

//! Planning automatic reframing (pure): which clips can follow a subject, and the change that
//! makes each one tracked from its asset's analyses (`se_domain::tracking`).

use std::collections::{BTreeMap, BTreeSet};

use se_domain::{
    AssetId, AssetKind, ClipTiming, DomainError, Flicks, Item, ItemId, ItemKind, ItemPatch,
    MediaClip, MediaClipPatch, Operation, Project, Reframe, Sequence, SequenceId, SubjectDetection,
    Track, TrackKind, TrackedFocus, TrackingRequest, TrackingSettings, UpdateItem, track_subject,
};
use se_protocol::{ReframeSkipReason, SkippedReframe};

use crate::AppError;

/// A clip automatic reframing can follow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReframableClip {
    /// The item.
    pub item_id: ItemId,
    /// Its media.
    pub asset_id: AssetId,
    /// Its sequence (whose frame the window takes).
    pub sequence_id: SequenceId,
}

/// The faces and shot changes of one asset.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetAnalyses {
    /// Faces of the sampled pictures.
    pub detection: SubjectDetection,
    /// Shot changes (source time); empty when the scene detection could not run.
    pub scene_changes: Vec<Flicks>,
}

/// Splits `item_ids` (duplicates ignored, order kept) into clips that can be reframed and the
/// others, with why.
///
/// # Errors
/// [`DomainError::ItemNotFound`] for an id the project lacks.
pub fn select_clips(
    project: &Project,
    item_ids: &[ItemId],
) -> Result<(Vec<ReframableClip>, Vec<SkippedReframe>), AppError> {
    let mut seen = BTreeSet::new();
    let (mut clips, mut skipped) = (Vec::new(), Vec::new());
    for &item_id in item_ids {
        if !seen.insert(item_id) {
            continue;
        }
        let (sequence, track, item) =
            find_item(project, item_id).ok_or(DomainError::ItemNotFound { item: item_id })?;
        match reframable(project, track, item) {
            Ok(asset_id) => clips.push(ReframableClip {
                item_id,
                asset_id,
                sequence_id: sequence.id,
            }),
            Err(reason) => skipped.push(SkippedReframe { item_id, reason }),
        }
    }
    Ok((clips, skipped))
}

/// Every media clip on a video track of `sequence`, in track then time order.
#[must_use]
pub fn video_clips(sequence: &Sequence) -> Vec<ItemId> {
    sequence
        .tracks
        .iter()
        .filter(|track| track.kind == TrackKind::Video)
        .flat_map(|track| &track.items)
        .filter(|item| matches!(item.kind, ItemKind::MediaClip(_)))
        .map(|item| item.id)
        .collect()
}

/// The asset of a clip that can follow its subject, or why it cannot.
fn reframable(project: &Project, track: &Track, item: &Item) -> Result<AssetId, ReframeSkipReason> {
    let ItemKind::MediaClip(clip) = &item.kind else {
        return Err(ReframeSkipReason::NotAVideoClip);
    };
    if track.kind != TrackKind::Video {
        return Err(ReframeSkipReason::NotAVideoClip);
    }
    if track.locked {
        return Err(ReframeSkipReason::LockedTrack);
    }
    let asset = project
        .assets
        .iter()
        .find(|asset| asset.id == clip.asset_id)
        .ok_or(ReframeSkipReason::NoPicture)?;
    if asset.missing {
        return Err(ReframeSkipReason::MediaMissing);
    }
    let has_picture = asset.kind == AssetKind::Video
        && asset
            .media_info
            .as_ref()
            .and_then(|info| info.video_stream.as_ref())
            .and_then(se_domain::VideoStreamInfo::display_aspect)
            .is_some();
    if has_picture {
        Ok(asset.id)
    } else {
        Err(ReframeSkipReason::NoPicture)
    }
}

/// The tracked reframe of `clip` from its asset's `analyses`, with what the tracking saw;
/// `None` when the clip, its sequence or its picture size is gone.
#[must_use]
pub fn tracked_reframe(
    project: &Project,
    clip: &ReframableClip,
    analyses: &AssetAnalyses,
    settings: TrackingSettings,
) -> Option<(Reframe, TrackedFocus)> {
    let (sequence, _, item) = find_item(project, clip.item_id)?;
    let ItemKind::MediaClip(media) = &item.kind else {
        return None;
    };
    let picture_aspect = project
        .assets
        .iter()
        .find(|asset| asset.id == media.asset_id)?
        .media_info
        .as_ref()?
        .video_stream
        .as_ref()?
        .display_aspect()?;
    let format = &sequence.format;
    let frame_aspect = f64::from(format.width) / f64::from(format.height.max(1));
    let tracked = track_subject(&TrackingRequest {
        detection: &analyses.detection,
        scene_changes: &analyses.scene_changes,
        picture_aspect,
        frame_aspect,
        clip: timing(media, item, sequence),
        settings,
    });
    let reframe = Reframe::Tracked {
        focus: tracked.focus.clone(),
        tracking: settings,
    };
    Some((reframe, tracked))
}

/// The operations making every clip of `clips` whose asset has analyses tracked.
#[must_use]
pub fn reframe_operations(
    project: &Project,
    clips: &[ReframableClip],
    analyses: &BTreeMap<AssetId, AssetAnalyses>,
    settings: TrackingSettings,
) -> Vec<Operation> {
    clips
        .iter()
        .filter_map(|clip| {
            let analyses = analyses.get(&clip.asset_id)?;
            let (reframe, _) = tracked_reframe(project, clip, analyses, settings)?;
            Some(
                UpdateItem {
                    item: clip.item_id,
                    patch: ItemPatch::MediaClip(MediaClipPatch {
                        reframe: Some(reframe),
                        ..MediaClipPatch::default()
                    }),
                    expected_revision: None,
                }
                .into(),
            )
        })
        .collect()
}

fn timing(clip: &MediaClip, item: &Item, sequence: &Sequence) -> ClipTiming {
    ClipTiming {
        source_start: clip.source_start,
        speed: clip.speed,
        duration: item.timeline_range.duration(),
        frame_rate: sequence.format.frame_rate,
    }
}

fn find_item(project: &Project, id: ItemId) -> Option<(&Sequence, &Track, &Item)> {
    project.sequences.iter().find_map(|sequence| {
        sequence.tracks.iter().find_map(|track| {
            track
                .items
                .iter()
                .find(|item| item.id == id)
                .map(|item| (sequence, track, item))
        })
    })
}

#[cfg(test)]
mod tests {
    use se_domain::{
        FaceBox, SubjectFrame,
        test_support::{example_ids, example_project},
    };

    use super::*;

    fn detection() -> SubjectDetection {
        SubjectDetection {
            model: "test".to_owned(),
            interval: Flicks::from_millis(200),
            frames: (0_u32..300)
                .map(|index| SubjectFrame {
                    time: Flicks::from_millis(index * 200),
                    faces: vec![FaceBox {
                        x: 0.2,
                        y: 0.2,
                        width: 0.1,
                        height: 0.14,
                        score: 0.9,
                    }],
                })
                .collect(),
        }
    }

    fn every_item(project: &Project) -> Vec<ItemId> {
        project.items().map(|item| item.id).collect()
    }

    #[test]
    fn only_clips_of_pictures_on_unlocked_video_tracks_are_reframed() {
        let mut project = example_project();
        let ids = every_item(&project);
        let (clips, skipped) = select_clips(&project, &ids).unwrap();
        assert_ne!(clips, []);
        assert!(
            clips
                .iter()
                .all(|clip| clip.asset_id == example_ids::VIDEO_ASSET)
        );
        assert!(
            skipped
                .iter()
                .any(|skip| skip.reason == ReframeSkipReason::NotAVideoClip)
        );
        project.sequences[0].tracks[0].locked = true;
        let (_, skipped) = select_clips(&project, &ids).unwrap();
        assert!(
            skipped
                .iter()
                .any(|skip| skip.reason == ReframeSkipReason::LockedTrack)
        );
        project.assets[0].missing = true;
        project.sequences[0].tracks[0].locked = false;
        let (clips, skipped) = select_clips(&project, &ids).unwrap();
        assert_eq!(clips, []);
        assert!(
            skipped
                .iter()
                .any(|skip| skip.reason == ReframeSkipReason::MediaMissing)
        );
        let unknown = ItemId::from_test_index(999);
        assert!(select_clips(&project, &[unknown]).is_err());
    }

    #[test]
    fn reframed_clips_become_tracked_with_their_settings() {
        let project = example_project();
        let clips: Vec<ReframableClip> =
            select_clips(&project, &video_clips(&project.sequences[0]))
                .unwrap()
                .0;
        let analyses = BTreeMap::from([(
            example_ids::VIDEO_ASSET,
            AssetAnalyses {
                detection: detection(),
                scene_changes: Vec::new(),
            },
        )]);
        let settings = TrackingSettings::default();
        let ops = reframe_operations(&project, &clips, &analyses, settings);
        assert_eq!(ops.len(), clips.len());
        let applied = se_domain::apply_all(&project, &ops).unwrap();
        for clip in &clips {
            let item = applied
                .project
                .items()
                .find(|item| item.id == clip.item_id)
                .unwrap();
            let ItemKind::MediaClip(media) = &item.kind else {
                panic!("a media clip");
            };
            assert_eq!(media.reframe.tracking(), Some(&settings));
        }
    }
}

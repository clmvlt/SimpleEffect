//! Media dropped on a sequence (prompt 11, ADR-0021): clips placed one after the other, a video
//! with sound as two linked items, on given tracks or on new ones, in one user transaction.

use se_domain::{AddTrack, ItemId, SequenceId, TrackId, TrackKind};
use se_protocol::{PlaceMediaRequest, TrackTarget};

use super::{ProjectService, items::labels, tracks::next_track_name};
use crate::{
    AppError,
    edit_plan::{EditPlan, find_sequence, find_track, media_parts, place},
    new_project::empty_track,
};

impl ProjectService {
    /// Adds dropped media as clips placed one after the other from `request.start`; a video with
    /// sound becomes a picture and a sound item, linked. Returns the new items' ids.
    ///
    /// # Errors
    /// Not open, unknown sequence, asset or track, a track of the wrong kind, an asset of unknown
    /// duration, refusals of the document (locked track…), storage errors.
    pub async fn place_media(&self, request: &PlaceMediaRequest) -> Result<Vec<ItemId>, AppError> {
        let handle = self.handle(request.project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        let sequence = request.sequence_id;
        let mut targets = TrackTargets::new(request.video_track, request.audio_track);
        let mut placed = Vec::new();
        let mut start = request.start;
        for &asset_id in &request.asset_ids {
            let asset = view
                .project
                .assets
                .iter()
                .find(|asset| asset.id == asset_id)
                .ok_or(AppError::AssetNotFound {
                    project_id: request.project_id,
                    asset_id,
                })?;
            let parts = media_parts(asset, start, plan.ids())?;
            for (item, kind) in [
                (parts.video, TrackKind::Video),
                (parts.audio, TrackKind::Audio),
            ] {
                if let Some(item) = item {
                    let track = targets.resolve(&mut plan, sequence, kind)?;
                    placed.push((track, item));
                }
            }
            start = start
                .checked_add(parts.duration)
                .ok_or_else(|| AppError::InvalidEdit {
                    reason: "the dropped media would end beyond the supported duration".to_owned(),
                })?;
        }
        let new_ids = placed.iter().map(|(_, item)| item.id).collect();
        place(&mut plan, sequence, placed, request.mode)?;
        self.commit_plan(&handle, labels::ADD_CLIPS, plan).await?;
        Ok(new_ids)
    }
}

/// The tracks dropped media goes to: given ones are checked, new ones are added on first use
/// (one per kind).
struct TrackTargets {
    video: TrackTarget,
    audio: TrackTarget,
    resolved_video: Option<TrackId>,
    resolved_audio: Option<TrackId>,
}

impl TrackTargets {
    const fn new(video: TrackTarget, audio: TrackTarget) -> Self {
        Self {
            video,
            audio,
            resolved_video: None,
            resolved_audio: None,
        }
    }

    /// The track receiving parts of `kind`, adding it to the plan when it is new.
    fn resolve(
        &mut self,
        plan: &mut EditPlan<'_>,
        sequence_id: SequenceId,
        kind: TrackKind,
    ) -> Result<TrackId, AppError> {
        let (target, resolved) = match kind {
            TrackKind::Video => (self.video, &mut self.resolved_video),
            TrackKind::Audio => (self.audio, &mut self.resolved_audio),
        };
        if let Some(track) = *resolved {
            return Ok(track);
        }
        let track = match target {
            TrackTarget::Existing { track_id } => {
                existing_target(plan, sequence_id, track_id, kind)?
            }
            TrackTarget::New => {
                let name = next_track_name(find_sequence(plan.project()?, sequence_id)?, kind);
                let track = empty_track(plan.ids(), &name, kind)?;
                let track_id = track.id;
                plan.push(AddTrack {
                    sequence: sequence_id,
                    index: None,
                    track,
                });
                track_id
            }
        };
        *resolved = Some(track);
        Ok(track)
    }
}

/// `track_id`, checked to be a track of `kind` in the sequence.
fn existing_target(
    plan: &mut EditPlan<'_>,
    sequence_id: SequenceId,
    track_id: TrackId,
    kind: TrackKind,
) -> Result<TrackId, AppError> {
    let (sequence, track) = find_track(plan.project()?, track_id)?;
    if sequence.id != sequence_id || track.kind != kind {
        return Err(AppError::InvalidEdit {
            reason: format!(
                "track {} is not a {} track of sequence {sequence_id}: drop pictures on video \
                 tracks and sound on audio tracks",
                track.name,
                match kind {
                    TrackKind::Video => "video",
                    TrackKind::Audio => "audio",
                }
            ),
        });
    }
    Ok(track_id)
}

//! Item edits of open projects (prompt 11, ADR-0021): dropped media, moves, copies, pastes,
//! deletes (with or without ripple), links. Each is one user transaction planned with
//! [`EditPlan`] against the current document, so one undo step.

use se_domain::{
    AddTrack, Item, ItemId, LinkGroupId, LinkItems, ProjectId, RemoveItem, SequenceId, TrackId,
    TrackKind, UnlinkItems,
};
use se_protocol::{ItemPlacement, PastedItem, PlaceMediaRequest, PlacementMode, TrackTarget};

use super::{ProjectService, tracks::next_track_name};
use crate::{
    AppError,
    edit_plan::{
        EditPlan, copies_of, distinct, find_item, find_sequence, find_track, lift, media_parts,
        place, ripple_delete, sequence_of, starting_at,
    },
    new_project::empty_track,
};

/// Labels of the item transactions (undo menu). Stable identifiers: the renderer translates them
/// (`apps/desktop/src/app/editor/history-label.ts`).
pub mod labels {
    /// Media was dropped on a sequence.
    pub const ADD_CLIPS: &str = "Add clips";
    /// Items were moved.
    pub const MOVE: &str = "Move items";
    /// Items were duplicated.
    pub const DUPLICATE: &str = "Duplicate items";
    /// Clipboard items were pasted.
    pub const PASTE: &str = "Paste items";
    /// Items were deleted, leaving gaps.
    pub const DELETE: &str = "Delete items";
    /// Items were deleted and their time closed.
    pub const RIPPLE_DELETE: &str = "Ripple delete";
    /// Items were linked.
    pub const LINK: &str = "Link items";
    /// Items were unlinked.
    pub const UNLINK: &str = "Unlink items";
    /// Item edges were moved.
    pub const TRIM: &str = "Trim items";
    /// Item edges were moved, shifting what follows.
    pub const RIPPLE_TRIM: &str = "Ripple trim";
    /// Items were cut in two.
    pub const SPLIT: &str = "Split items";
    /// A gap was closed.
    pub const CLOSE_GAP: &str = "Close gap";
    /// A gap was lengthened or shortened.
    pub const RESIZE_GAP: &str = "Resize gap";
}

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

    /// Moves items to new tracks and starts in one sequence, with `mode` for what they land on.
    ///
    /// # Errors
    /// Not open, nothing listed or an item twice, unknown items or tracks, several sequences,
    /// refusals of the document, storage errors.
    pub async fn move_items(
        &self,
        project_id: ProjectId,
        moves: &[ItemPlacement],
        mode: PlacementMode,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        distinct(moves.iter().map(|placement| placement.item_id))?;
        let sequence = placements_sequence(&mut plan, moves)?;
        let item_ids: Vec<ItemId> = moves.iter().map(|placement| placement.item_id).collect();
        let lifted = lift(&mut plan, &item_ids)?;
        let placed = lifted
            .into_iter()
            .zip(moves)
            .map(|((_, item), placement)| {
                Ok((placement.track_id, starting_at(item, placement.start)?))
            })
            .collect::<Result<Vec<_>, AppError>>()?;
        place(&mut plan, sequence, placed, mode)?;
        self.commit_plan(&handle, labels::MOVE, plan).await
    }

    /// Copies items of one sequence to new places, with `mode` for what the copies land on.
    /// Returns the copies' ids in the order of `copies`.
    ///
    /// # Errors
    /// Like [`Self::move_items`].
    pub async fn duplicate_items(
        &self,
        project_id: ProjectId,
        copies: &[ItemPlacement],
        mode: PlacementMode,
    ) -> Result<Vec<ItemId>, AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        distinct(copies.iter().map(|placement| placement.item_id))?;
        let sequence = placements_sequence(&mut plan, copies)?;
        let project = plan.project()?;
        let originals = copies
            .iter()
            .map(|placement| {
                let item = find_item(project, placement.item_id)?.item.clone();
                starting_at(item, placement.start)
            })
            .collect::<Result<Vec<_>, AppError>>()?;
        let tracks = copies.iter().map(|placement| placement.track_id);
        let new_items = copies_of(originals, plan.ids())?;
        let new_ids = new_items.iter().map(|item| item.id).collect();
        place(&mut plan, sequence, tracks.zip(new_items).collect(), mode)?;
        self.commit_plan(&handle, labels::DUPLICATE, plan).await?;
        Ok(new_ids)
    }

    /// Adds copies of clipboard items (given by content, already at their new starts) to tracks
    /// of one sequence. Returns the copies' ids in the order of `items`.
    ///
    /// # Errors
    /// Not open, nothing listed, unknown or locked tracks, several sequences, unknown assets,
    /// refusals of the document, storage errors.
    pub async fn paste_items(
        &self,
        project_id: ProjectId,
        items: &[PastedItem],
        mode: PlacementMode,
    ) -> Result<Vec<ItemId>, AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        let tracks = items.iter().map(|pasted| pasted.track_id);
        let sequence = sequence_of(plan.project()?, [], tracks)?;
        let contents: Vec<Item> = items.iter().map(|pasted| pasted.item.clone()).collect();
        let new_items = copies_of(contents, plan.ids())?;
        let new_ids = new_items.iter().map(|item| item.id).collect();
        let tracks = items.iter().map(|pasted| pasted.track_id);
        place(&mut plan, sequence, tracks.zip(new_items).collect(), mode)?;
        self.commit_plan(&handle, labels::PASTE, plan).await?;
        Ok(new_ids)
    }

    /// Deletes items of one sequence; with `ripple`, closes the time they took on their tracks
    /// and on the tracks of items linked to what follows.
    ///
    /// # Errors
    /// Not open, nothing listed or an item twice, unknown items, several sequences,
    /// [`AppError::RippleBlocked`], locked tracks, storage errors.
    pub async fn delete_items(
        &self,
        project_id: ProjectId,
        item_ids: &[ItemId],
        ripple: bool,
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        let items = distinct(item_ids.iter().copied())?;
        let sequence = sequence_of(plan.project()?, items.iter().copied(), [])?;
        if ripple {
            ripple_delete(&mut plan, sequence, &items)?;
        } else {
            for &item in &items {
                plan.push(RemoveItem {
                    item,
                    expected_revision: None,
                });
            }
        }
        let label = if ripple {
            labels::RIPPLE_DELETE
        } else {
            labels::DELETE
        };
        self.commit_plan(&handle, label, plan).await
    }

    /// Puts at least two items of one sequence in a new link group (they then select and move
    /// together). Returns the group.
    ///
    /// # Errors
    /// Not open, fewer than two items, unknown items, several sequences, storage errors.
    pub async fn link_items(
        &self,
        project_id: ProjectId,
        item_ids: &[ItemId],
    ) -> Result<LinkGroupId, AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        let items = distinct(item_ids.iter().copied())?;
        if items.len() < 2 {
            return Err(AppError::InvalidEdit {
                reason: "linking needs at least two items: select the picture and the sound to \
                         link"
                    .to_owned(),
            });
        }
        sequence_of(plan.project()?, items.iter().copied(), [])?;
        let group = plan.ids().link_group()?;
        plan.push(LinkItems {
            items: items.into_iter().collect(),
            group,
        });
        self.commit_plan(&handle, labels::LINK, plan).await?;
        Ok(group)
    }

    /// Takes items out of their link groups.
    ///
    /// # Errors
    /// Not open, nothing listed or an item twice, unknown items, storage errors.
    pub async fn unlink_items(
        &self,
        project_id: ProjectId,
        item_ids: &[ItemId],
    ) -> Result<(), AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        let items = distinct(item_ids.iter().copied())?;
        plan.push(UnlinkItems {
            items: items.into_iter().collect(),
        });
        self.commit_plan(&handle, labels::UNLINK, plan).await
    }
}

/// The one sequence holding the items and target tracks of `placements`.
fn placements_sequence(
    plan: &mut EditPlan<'_>,
    placements: &[ItemPlacement],
) -> Result<SequenceId, AppError> {
    sequence_of(
        plan.project()?,
        placements.iter().map(|placement| placement.item_id),
        placements.iter().map(|placement| placement.track_id),
    )
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

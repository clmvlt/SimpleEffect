//! Automatic reframing by agents (prompt 26, ADR-0037): clips of the scope made to follow their
//! subject, and versions of the task's sequence in another format whose clips follow theirs.
//! The analyses are waited for before planning (se-mcp, through `ReframeService::analyses_for`);
//! the geometry is computed on the draft, so it follows the document the edit applies to.

use se_domain::{AddSequence, FormatAdaptation, ItemId, SequenceId, sequence_version};

use super::{ReframeClips, ReframedVersion, planner::AgentPlanner};
use crate::{
    AppError,
    project_service::CopyIds,
    reframing::{reframe_operations, select_clips, video_clips},
};

/// Plans the reframing of `request`'s clips; refuses a clip that cannot follow a subject or
/// whose media has no analyses.
///
/// # Errors
/// [`AppError::InvalidEdit`] naming the clip and why; unknown items.
pub(super) fn plan_reframe_clips(
    planner: &mut AgentPlanner<'_>,
    request: &ReframeClips,
) -> Result<(), AppError> {
    let items: Vec<ItemId> = request
        .items
        .iter()
        .map(|item| planner.resolve(*item))
        .collect::<Result<_, _>>()?;
    let project = planner.plan.project()?.clone();
    let (clips, skipped) = select_clips(&project, &items)?;
    if let Some(skip) = skipped.first() {
        return Err(AppError::InvalidEdit {
            reason: format!(
                "item {} cannot follow a subject ({}): give media clips of video tracks whose \
                 media has a picture",
                skip.item_id,
                skip_reason(skip.reason)
            ),
        });
    }
    if let Some(clip) = clips
        .iter()
        .find(|clip| !request.analyses.contains_key(&clip.asset_id))
    {
        return Err(AppError::InvalidEdit {
            reason: format!(
                "the faces of the media of clip {} are not analysed yet: call auto_reframe again",
                clip.item_id
            ),
        });
    }
    for operation in reframe_operations(&project, &clips, &request.analyses, request.settings) {
        planner.plan.push(operation);
    }
    Ok(())
}

/// Plans a version of the task's sequence in `request.format` placed after it, every clip of its
/// video tracks filling the frame and following its subject; returns its id.
///
/// # Errors
/// The copy is refused (frame rate change of a sequence holding items), ids, the document.
pub(super) fn plan_reframed_version(
    planner: &mut AgentPlanner<'_>,
    request: &ReframedVersion,
) -> Result<SequenceId, AppError> {
    let project = planner.plan.project()?.clone();
    let source_id = planner.agent.sequence_id();
    let source = project
        .sequences
        .iter()
        .find(|sequence| sequence.id == source_id)
        .ok_or_else(|| AppError::InvalidEdit {
            reason: "the task's sequence no longer exists".to_owned(),
        })?;
    let adaptation = FormatAdaptation {
        clip_reframe: se_domain::ClipReframePolicy::Fill,
        fit_text_widths: true,
    };
    let mut ids = CopyIds(planner.plan.ids());
    let copy = sequence_version(
        source,
        request.name.clone(),
        request.format,
        adaptation,
        &mut ids,
    )?;
    let copy_id = copy.id;
    let index = project
        .sequences
        .iter()
        .position(|sequence| sequence.id == source_id)
        .map(|position| position + 1);
    let clips = video_clips(&copy);
    planner.plan.push(AddSequence {
        index,
        sequence: copy,
    });
    let draft = planner.plan.project()?.clone();
    let (reframable, _) = select_clips(&draft, &clips)?;
    for operation in reframe_operations(&draft, &reframable, &request.analyses, request.settings) {
        planner.plan.push(operation);
    }
    Ok(copy_id)
}

/// A reason in words, for agents.
const fn skip_reason(reason: se_protocol::ReframeSkipReason) -> &'static str {
    match reason {
        se_protocol::ReframeSkipReason::NotAVideoClip => "not a media clip on a video track",
        se_protocol::ReframeSkipReason::NoPicture => "its media has no picture of known size",
        se_protocol::ReframeSkipReason::MediaMissing => "its media file is missing",
        se_protocol::ReframeSkipReason::LockedTrack => "its track is locked",
    }
}

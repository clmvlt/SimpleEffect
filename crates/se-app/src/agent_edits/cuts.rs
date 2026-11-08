//! Smart cuts of AI agents (prompt 22, ADR-0032): the same resolution and planning as the user's
//! cuts, limited to the task's scope with the « cut » rule ([`super::cut_scope`]), committed as
//! one AI transaction recorded in the task's changeset.

use std::sync::Arc;

use se_domain::{Project, TrackKind};
use se_protocol::CutSummary;

use super::{AgentEditService, MAX_ATTEMPTS, cut_scope::check_cut_scope, scope::track_allowed};
use crate::{
    AppError,
    agent_access::AgentContext,
    edit_plan::{EditPlan, find_sequence},
    new_ids::NewIds,
    smart_cuts::{CutMedia, CutRequest, CutTarget, apply::plan_cut, resolve_with_tracks},
};

/// What an agent's cut did.
#[derive(Debug, Clone)]
pub struct CutOutcome {
    /// The cuts made (sequence time before the cut), the time gained, the tracks edited.
    pub summary: CutSummary,
    /// The project's revision after the cut; `None` when there was nothing to cut.
    pub revision: Option<u64>,
    /// The document after the cut (or as it was).
    pub project: Arc<Project>,
}

impl AgentEditService {
    /// Resolves `request` on the task's sequence, inside its scope (cuts never leave the scope's
    /// range; silences are read on the scope's sound tracks by default), checks the « cut » scope
    /// rule and commits the cuts as one transaction. Nothing is committed when nothing matched.
    ///
    /// # Errors
    /// [`AppError::OutOfScope`], unknown tracks, locked tracks, the domain's refusals, storage
    /// errors; [`AppError::InvalidEdit`] when the project kept changing.
    pub async fn apply_cuts(
        &self,
        agent: &AgentContext,
        label: &str,
        request: &CutRequest,
        media: &CutMedia,
    ) -> Result<CutOutcome, AppError> {
        let handle = self.projects.handle(agent.project_id).await?;
        for _ in 0..MAX_ATTEMPTS {
            let view = handle.view().await?;
            let base = &view.project;
            let request = scoped(agent, base, request)?;
            let (resolved, mut tracks) = resolve_with_tracks(base, &request, media)?;
            // Caption tracks follow the cut only when the task may edit them.
            tracks.retain(|&track| {
                track_allowed(&agent.scope, track) || !is_caption_track_of(base, track)
            });
            let summary = resolved.summary(&tracks);
            if resolved.cuts.is_empty() {
                return Ok(CutOutcome {
                    summary,
                    revision: None,
                    project: Arc::clone(&view.project),
                });
            }
            check_cut_scope(
                &agent.scope,
                base,
                request.sequence_id,
                &resolved.cuts,
                &tracks,
            )?;
            let mut plan = EditPlan::new(base, NewIds(self.ids.as_ref()));
            plan_cut(&mut plan, request.sequence_id, &resolved.cuts, &tracks)?;
            let transaction = self.transaction(agent, label, plan.finish()?)?;
            let Some(commit) = handle.apply_ai_on(transaction, base.revision).await? else {
                continue;
            };
            self.note_changeset(agent, &commit).await;
            return Ok(CutOutcome {
                summary,
                revision: Some(commit.revision),
                project: commit.project,
            });
        }
        Err(AppError::InvalidEdit {
            reason: "the project kept changing while the cut was planned: read it again and retry"
                .to_owned(),
        })
    }
}

/// `request` on the task's sequence, limited to its range, silences read on its sound tracks
/// when it has tracks and the request names none.
fn scoped(
    agent: &AgentContext,
    project: &Project,
    request: &CutRequest,
) -> Result<CutRequest, AppError> {
    let scope = &agent.scope;
    let limit = match (scope.range, request.limit) {
        (Some(range), Some(limit)) => range.intersection(limit).or(Some(range)),
        (range, limit) => range.or(limit),
    };
    let mut scoped = CutRequest {
        sequence_id: scope.sequence_id,
        limit,
        ..request.clone()
    };
    if let (
        CutTarget::Silences {
            tracks: tracks @ None,
            ..
        },
        Some(allowed),
    ) = (&mut scoped.target, &scope.tracks)
    {
        let sequence = find_sequence(project, scope.sequence_id)?;
        let sound: Vec<_> = sequence
            .tracks
            .iter()
            .filter(|track| {
                track.kind == TrackKind::Audio && !track.muted && allowed.contains(&track.id)
            })
            .map(|track| track.id)
            .collect();
        *tracks = Some(sound);
    }
    Ok(scoped)
}

/// Whether `track` of `project` is a caption track.
fn is_caption_track_of(project: &Project, track: se_domain::TrackId) -> bool {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .any(|candidate| {
            candidate.id == track && crate::captions::track::is_caption_track(candidate)
        })
}

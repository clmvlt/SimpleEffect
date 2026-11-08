//! Smart cuts committed by the user (prompt 22, ADR-0032): one ripple transaction each.

use se_domain::ProjectId;
use se_protocol::CutSummary;

use super::ProjectService;
use crate::{
    AppError,
    edit_plan::EditPlan,
    smart_cuts::{CutMedia, CutRequest, apply::plan_cut, resolve_with_tracks},
};

impl ProjectService {
    /// Resolves `request` on the current document and commits its cuts as one user transaction
    /// labelled `label`; nothing is committed when there is nothing to cut.
    ///
    /// # Errors
    /// Not open, unknown sequence or tracks, locked tracks, the domain's refusals, storage errors.
    pub(crate) async fn apply_cut(
        &self,
        project_id: ProjectId,
        label: &str,
        request: &CutRequest,
        media: &CutMedia,
    ) -> Result<CutSummary, AppError> {
        let handle = self.handle(project_id).await?;
        let view = handle.view().await?;
        let (resolved, tracks) = resolve_with_tracks(&view.project, request, media)?;
        let summary = resolved.summary(&tracks);
        if resolved.cuts.is_empty() {
            return Ok(summary);
        }
        let mut plan = EditPlan::new(&view.project, self.new_ids());
        plan_cut(&mut plan, request.sequence_id, &resolved.cuts, &tracks)?;
        self.commit_plan(&handle, label, plan).await?;
        Ok(summary)
    }
}

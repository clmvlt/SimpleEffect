//! Motion templates (prompt 24, ADR-0035): the renderer's verdict on a template module an agent
//! wrote, and the removal of templates no item uses.

use se_domain::{ProjectId, TemplateId};
use se_protocol::{CommandResult, ProtocolError, TemplateCheckOutcome};

use super::HandlerContext;

/// `reportTemplateCheck`: the agent waiting for `request_id` gets `outcome`.
///
/// # Errors
/// `TEMPLATE_CHECK_NOT_FOUND` when nothing waits for it any more.
pub fn report_template_check(
    context: &HandlerContext,
    request_id: &str,
    outcome: TemplateCheckOutcome,
) -> Result<CommandResult, ProtocolError> {
    context.template_checks.deliver(request_id, outcome)?;
    Ok(CommandResult::ReportTemplateCheck)
}

/// `removeTemplate`: one undoable step.
///
/// # Errors
/// Not open, unknown template, a template still used (`EDIT_REJECTED`, domain code
/// `TEMPLATE_IN_USE`), storage errors.
pub async fn remove_template(
    context: &HandlerContext,
    project_id: ProjectId,
    template_id: TemplateId,
) -> Result<CommandResult, ProtocolError> {
    context
        .projects
        .remove_template(project_id, template_id)
        .await?;
    Ok(CommandResult::RemoveTemplate)
}

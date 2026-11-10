//! « Exporter un diagnostic » (prompt 30): gathers what the services know about the machine and
//! hands it to se-app's `DiagnosticsService`, which masks it and writes the zip.

use se_app::DiagnosticsReport;
use se_protocol::{ClientDiagnostics, CommandResult, ProtocolError};

use super::{HandlerContext, absolute_path, system::core_info};

/// `exportDiagnostics`.
pub async fn export_diagnostics(
    context: &HandlerContext,
    path: &str,
    client: &ClientDiagnostics,
) -> Result<CommandResult, ProtocolError> {
    let path = absolute_path(path)?;
    let report = DiagnosticsReport {
        core: core_info(context),
        ai_status: context.tasks.ai_status().await,
        ai_settings: context.tasks.ai_settings(),
        media_tools: context.media.tools_status().await,
        media_settings: context.media.media_settings().await?,
        transcription: context.transcription.status().await?,
        vision: context.reframing.status().await,
        client: client.clone(),
    };
    let summary = context.diagnostics.export(path, &report).await?;
    Ok(CommandResult::ExportDiagnostics(summary))
}

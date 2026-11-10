//! Video exports (prompt 27, ADR-0038): starting, cancelling and listing exports, the export
//! dialog's preferences, and the renderer's reports on an export it renders (its uploads go
//! through `PUT /media/exports/…`, `export_routes.rs`).

use se_protocol::{Command, CommandResult, ErrorCode, ProtocolError};

use super::HandlerContext;

/// The export commands, one arm each.
///
/// # Errors
/// The handler's error; `INTERNAL` for a command of another group.
pub async fn dispatch(
    context: &HandlerContext,
    command: &Command,
) -> Result<CommandResult, ProtocolError> {
    let exports = &context.exports;
    match command {
        Command::StartExport(request) => Ok(CommandResult::StartExport {
            export: exports.start(request.clone()).await?,
        }),
        Command::CancelExport { export_id } => {
            exports.cancel(*export_id)?;
            Ok(CommandResult::CancelExport)
        }
        Command::ListExports => Ok(CommandResult::ListExports {
            exports: exports.list(),
        }),
        Command::ClearFinishedExports => {
            exports.clear_finished();
            Ok(CommandResult::ClearFinishedExports)
        }
        Command::ReportExportProgress {
            export_id,
            done,
            total,
        } => {
            exports.report_progress(*export_id, *done, *total).await?;
            Ok(CommandResult::ReportExportProgress)
        }
        Command::FinishExportRender { export_id } => {
            exports.finish_render(*export_id).await?;
            Ok(CommandResult::FinishExportRender)
        }
        Command::FailExportRender { export_id, message } => {
            exports.fail_render(*export_id, message.clone()).await?;
            Ok(CommandResult::FailExportRender)
        }
        Command::GetExportPreferences => Ok(CommandResult::GetExportPreferences(
            exports.preferences().await?,
        )),
        Command::UpdateExportPreferences(preferences) => {
            exports.update_preferences(preferences).await?;
            Ok(CommandResult::UpdateExportPreferences)
        }
        _ => Err(ProtocolError::new(
            ErrorCode::Internal,
            "a non-export command reached the export dispatcher",
        )),
    }
}

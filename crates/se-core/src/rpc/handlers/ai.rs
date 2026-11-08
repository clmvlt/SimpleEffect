//! AI provider commands (prompt 17): status and settings.

use se_protocol::{AiSettings, CommandResult, ProtocolError};

use super::HandlerContext;

/// `getAiStatus`: checked once, then kept (refreshed by `refreshAiStatus` and failed runs).
pub async fn get_ai_status(context: &HandlerContext) -> CommandResult {
    CommandResult::GetAiStatus {
        status: context.tasks.ai_status().await,
    }
}

/// `refreshAiStatus`.
pub async fn refresh_ai_status(context: &HandlerContext) -> CommandResult {
    CommandResult::RefreshAiStatus {
        status: context.tasks.refresh_ai_status().await,
    }
}

/// `getAiSettings`.
pub fn get_ai_settings(context: &HandlerContext) -> CommandResult {
    CommandResult::GetAiSettings {
        settings: context.tasks.ai_settings(),
    }
}

/// `updateAiSettings`.
pub async fn update_ai_settings(
    context: &HandlerContext,
    settings: &AiSettings,
) -> Result<CommandResult, ProtocolError> {
    let settings = context.tasks.update_ai_settings(settings.clone()).await?;
    Ok(CommandResult::UpdateAiSettings { settings })
}

#[cfg(test)]
mod tests {
    use se_protocol::{ErrorCode, InstallationStatus};

    use super::*;
    use crate::rpc::router::testing::context;

    #[tokio::test]
    async fn the_status_and_settings_come_from_the_task_service() {
        let context = context();
        let CommandResult::GetAiStatus { status } = get_ai_status(&context).await else {
            panic!("getAiStatus must answer with the status");
        };
        assert!(matches!(
            status.installation,
            InstallationStatus::Installed { .. }
        ));
        let CommandResult::GetAiSettings { mut settings } = get_ai_settings(&context) else {
            panic!("getAiSettings must answer with the settings");
        };
        settings.max_concurrent_tasks = 9;
        let refused = update_ai_settings(&context, &settings).await;
        assert_eq!(refused.unwrap_err().code, ErrorCode::InvalidRequest);
    }
}

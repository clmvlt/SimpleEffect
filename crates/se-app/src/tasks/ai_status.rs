//! What is known about AI providers: checking one (installed? logged in?), keeping the answer,
//! publishing `aiStatusChanged` when it changes, and refusing tasks a provider cannot run.

use se_protocol::{
    AgentFailureKind, AiStatus, AuthStatus, CoreEvent, InstallationStatus, ProviderId,
};

use super::{Shared, lock};
use crate::{
    AppError,
    ports::{AgentError, AgentProvider},
};

impl Shared {
    /// Checks the provider now, keeps and publishes the answer.
    pub(super) async fn refresh_status(&self, provider_id: &ProviderId) -> AiStatus {
        let status = match self.provider(provider_id) {
            Some(provider) => check(provider.as_ref()).await,
            None => unknown_provider(provider_id),
        };
        self.remember_status(status.clone());
        status
    }

    /// Fails unless the provider can run tasks (checking it the first time). An unknown login
    /// state is let through: the run will tell.
    pub(super) async fn ensure_usable(&self, provider_id: &ProviderId) -> Result<(), AppError> {
        let known = lock(&self.statuses).get(provider_id).cloned();
        let status = match known {
            Some(status) => status,
            None => self.refresh_status(provider_id).await,
        };
        let reason = match (&status.installation, &status.auth) {
            (
                InstallationStatus::NotInstalled { message }
                | InstallationStatus::Unusable { message, .. },
                _,
            ) => message.clone(),
            (_, AuthStatus::LoggedOut) => format!(
                "{} is not logged in: open a terminal, run `{}`, then type `{}`; then refresh the \
                 AI status in the settings",
                status.provider.name, status.provider.command, status.provider.login_command
            ),
            _ => return Ok(()),
        };
        Err(AppError::AiUnavailable { reason })
    }

    /// Updates what is known after a run failed because the CLI vanished or logged out.
    pub(super) fn note_failure(
        &self,
        provider_id: &ProviderId,
        failure: AgentFailureKind,
        message: &str,
    ) {
        let known = lock(&self.statuses).get(provider_id).cloned();
        let Some(mut status) = known else {
            return;
        };
        match failure {
            AgentFailureKind::NotAuthenticated => status.auth = AuthStatus::LoggedOut,
            AgentFailureKind::NotInstalled => {
                status.installation = InstallationStatus::NotInstalled {
                    message: message.to_owned(),
                };
            }
            _ => return,
        }
        self.remember_status(status);
    }

    /// Keeps `status` and publishes it when it differs from what was known.
    fn remember_status(&self, status: AiStatus) {
        let previous = lock(&self.statuses).insert(status.provider.id.clone(), status.clone());
        if previous.as_ref() != Some(&status) {
            self.deps.events.publish(CoreEvent::AiStatusChanged(status));
        }
    }
}

/// Asks a provider where its CLI is, which version, and whether it is logged in.
async fn check(provider: &dyn AgentProvider) -> AiStatus {
    let installation = match provider.detect().await {
        Ok(found) => InstallationStatus::Installed {
            path: found.path.display().to_string(),
            version: found.version,
        },
        Err(AgentError::Unusable { path, message }) => InstallationStatus::Unusable {
            path: path.display().to_string(),
            message,
        },
        Err(error) => InstallationStatus::NotInstalled {
            message: error.to_string(),
        },
    };
    let auth = if matches!(installation, InstallationStatus::Installed { .. }) {
        match provider.auth_status().await {
            Ok(auth) => auth,
            Err(error) => AuthStatus::Unknown {
                reason: error.to_string(),
            },
        }
    } else {
        AuthStatus::Unknown {
            reason: "the CLI is not installed".to_owned(),
        }
    };
    AiStatus {
        provider: provider.info(),
        installation,
        auth,
        capabilities: provider.capabilities(),
    }
}

/// The status of a provider id nobody registered (a setting written by another build).
fn unknown_provider(provider_id: &ProviderId) -> AiStatus {
    AiStatus {
        provider: se_protocol::ProviderInfo {
            id: provider_id.clone(),
            name: provider_id.to_string(),
            command: provider_id.to_string(),
            login_command: String::new(),
            help_url: String::new(),
        },
        installation: InstallationStatus::NotInstalled {
            message: format!("this version of SimpleEffect has no AI provider {provider_id}"),
        },
        auth: AuthStatus::Unknown {
            reason: "unknown provider".to_owned(),
        },
        capabilities: se_protocol::ProviderCapabilities {
            resume_sessions: false,
            image_input: false,
            models: Vec::new(),
            effort_levels: Vec::new(),
        },
    }
}

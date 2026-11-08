//! [`ClaudeCodeProvider`]: Claude Code (`claude`) in headless mode, the first AI provider
//! (ADR-0003, ADR-0027). The only code that knows Claude Code: its flags, its stream-json output,
//! its error messages and where it is installed. Empirical findings (CLI version, flags kept,
//! isolation) are in `claude_code/CLAUDE.md`.

mod arguments;
mod auth;
mod environment;
mod failure;
mod identity;
mod locate;
mod platform;
mod run;
mod stream;

use std::{
    path::PathBuf,
    sync::{Mutex, PoisonError},
    time::Duration,
};

use async_trait::async_trait;
use se_app::ports::{AgentError, AgentProvider, AgentRun, AgentRunSpec, ProviderInstallation};
use se_protocol::{AuthStatus, ProviderCapabilities, ProviderInfo};

pub use identity::PROVIDER_ID;

use crate::process::run_command;

/// How long `claude --version` and `claude auth status` may take (the first start of a freshly
/// installed CLI is slow).
const QUICK_COMMAND_TIMEOUT: Duration = Duration::from_secs(20);

/// Runs agents with the Claude Code CLI installed on this machine.
#[derive(Debug, Default)]
pub struct ClaudeCodeProvider {
    /// The executable chosen in the settings (`None`: search it).
    configured: Mutex<Option<PathBuf>>,
}

impl ClaudeCodeProvider {
    /// A provider searching `claude` on the PATH and in the usual install folders.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn executable(&self) -> Result<PathBuf, AgentError> {
        let configured = self
            .configured
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        locate::resolve(configured.as_deref())
    }
}

#[async_trait]
impl AgentProvider for ClaudeCodeProvider {
    fn info(&self) -> ProviderInfo {
        identity::info()
    }

    fn capabilities(&self) -> ProviderCapabilities {
        identity::capabilities()
    }

    fn set_executable(&self, path: Option<PathBuf>) {
        *self
            .configured
            .lock()
            .unwrap_or_else(PoisonError::into_inner) = path;
    }

    async fn detect(&self) -> Result<ProviderInstallation, AgentError> {
        let path = self.executable()?;
        let unusable = |message: String| AgentError::Unusable {
            path: path.clone(),
            message,
        };
        let unset = environment::variables_to_remove();
        let output = run_command(
            &path,
            arguments::VERSION_ARGUMENTS,
            QUICK_COMMAND_TIMEOUT,
            &unset,
        )
        .await
        .map_err(|error| unusable(error.to_string()))?;
        if !output.success {
            return Err(unusable(format!(
                "`claude --version` failed ({}): {}",
                output.status, output.stderr
            )));
        }
        let version = auth::parse_version(&output.stdout).ok_or_else(|| {
            unusable(format!(
                "`claude --version` printed no version: {}",
                output.stdout.trim()
            ))
        })?;
        Ok(ProviderInstallation { path, version })
    }

    async fn auth_status(&self) -> Result<AuthStatus, AgentError> {
        let path = self.executable()?;
        let unset = environment::variables_to_remove();
        let output = run_command(
            &path,
            arguments::AUTH_STATUS_ARGUMENTS,
            QUICK_COMMAND_TIMEOUT,
            &unset,
        )
        .await?;
        // A logged-out CLI exits with an error but still prints its JSON answer.
        auth::parse_auth_status(&output.stdout)
    }

    async fn start(&self, spec: AgentRunSpec) -> Result<AgentRun, AgentError> {
        let path = self.executable()?;
        run::start(&path, spec).await
    }
}

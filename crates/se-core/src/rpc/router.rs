//! `RpcRouter`: one text frame in, one response frame out.

use se_protocol::{
    Command, CommandResult, ProtocolError, ResponseEnvelope, ResponseOutcome, ServerMessage,
};

use super::{
    encode::encode,
    handlers::{HandlerContext, editing, projects, system},
    request::{Rejection, parse_request},
};

/// Parses requests, dispatches each command to its handler and encodes the response.
///
/// Handlers are async; `connection.rs` runs each request in its own task, so a slow command
/// (opening a large project) never delays events or other requests. Requests of one client may
/// therefore complete in any order: a client that needs ordering waits for the previous answer.
pub struct RpcRouter {
    context: HandlerContext,
}

impl RpcRouter {
    /// A router whose handlers read `context`.
    pub fn new(context: HandlerContext) -> Self {
        Self { context }
    }

    /// Handles one text frame received on `/rpc` and returns the JSON response frame.
    pub async fn handle_text(&self, text: &str) -> String {
        let response = match parse_request(text) {
            Ok(request) => {
                tracing::debug!(id = request.id, command = ?request.command, "request");
                let outcome = match self.dispatch(&request.command).await {
                    Ok(result) => ResponseOutcome::Result(result),
                    Err(error) => {
                        tracing::info!(id = request.id, code = ?error.code, message = %error.message, "command failed");
                        ResponseOutcome::Error(error)
                    }
                };
                ResponseEnvelope {
                    id: Some(request.id),
                    outcome,
                }
            }
            Err(Rejection { id, error }) => {
                tracing::warn!(?id, code = ?error.code, message = %error.message, "request rejected");
                ResponseEnvelope {
                    id,
                    outcome: ResponseOutcome::Error(error),
                }
            }
        };
        encode(&ServerMessage::Response(response))
    }

    /// The one place mapping commands to handlers: add a match arm per new command.
    async fn dispatch(&self, command: &Command) -> Result<CommandResult, ProtocolError> {
        let context = &self.context;
        match command {
            Command::Ping => Ok(system::ping()),
            Command::GetCoreInfo => Ok(system::get_core_info(context)),
            Command::GetNewProjectDefaults => projects::get_new_project_defaults(context).await,
            Command::CreateProject(request) => projects::create_project(context, request).await,
            Command::OpenProject { path } => projects::open_project(context, path).await,
            Command::CloseProject { project_id } => {
                projects::close_project(context, *project_id).await
            }
            Command::ListRecentProjects => projects::list_recent_projects(context).await,
            Command::RenameProject { path, name } => {
                projects::rename_project(context, path, name).await
            }
            Command::DuplicateProject { path, name } => {
                projects::duplicate_project(context, path, name).await
            }
            Command::TrashProject { path } => projects::trash_project(context, path).await,
            Command::GetProjectSnapshot { project_id } => {
                editing::get_project_snapshot(context, *project_id).await
            }
            Command::CreateSequence {
                project_id,
                name,
                format,
            } => editing::create_sequence(context, *project_id, name, *format).await,
            Command::Undo { project_id } => editing::undo(context, *project_id).await,
            Command::Redo { project_id } => editing::redo(context, *project_id).await,
        }
    }
}

/// A handler context over in-memory ports, for unit tests.
#[cfg(test)]
pub mod testing {
    use std::{path::PathBuf, sync::Arc, time::Duration};

    use se_app::{
        ProjectService, ProjectServiceConfig, ProjectServiceDeps,
        test_support::{
            FixedClock, InMemoryAppLibrary, InMemoryProjectRepository, RecordingEventSink,
            SequentialIds,
        },
    };

    use crate::{app_paths::AppPaths, rpc::handlers::HandlerContext};

    /// Version `1.2.3`, folders under `data`, projects in memory.
    pub fn context() -> HandlerContext {
        let deps = ProjectServiceDeps {
            repository: Arc::new(InMemoryProjectRepository::default()),
            library: Arc::new(InMemoryAppLibrary::default()),
            events: Arc::new(RecordingEventSink::default()),
            clock: Arc::new(FixedClock::default()),
            ids: Arc::new(SequentialIds::default()),
        };
        let config = ProjectServiceConfig {
            default_parent_dir: PathBuf::from("/projects"),
            save_delay: Duration::from_secs(2),
        };
        HandlerContext {
            version: "1.2.3",
            paths: AppPaths {
                data_dir: PathBuf::from("data"),
                logs_dir: PathBuf::from("data").join("logs"),
            },
            projects: Arc::new(ProjectService::new(deps, config)),
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::*;

    async fn reply(text: &str) -> Value {
        let router = RpcRouter::new(testing::context());
        serde_json::from_str(&router.handle_text(text).await).unwrap()
    }

    #[tokio::test]
    async fn answers_ping_with_a_ping_result() {
        let router = RpcRouter::new(testing::context());
        assert_eq!(
            router
                .handle_text(r#"{"type":"request","id":7,"command":{"kind":"ping"}}"#)
                .await,
            r#"{"type":"response","id":7,"result":{"kind":"ping"}}"#
        );
    }

    #[tokio::test]
    async fn answers_get_core_info_with_the_context() {
        let reply = reply(r#"{"type":"request","id":8,"command":{"kind":"getCoreInfo"}}"#).await;
        assert_eq!(reply["id"], 8);
        assert_eq!(reply["result"]["kind"], "getCoreInfo");
        assert_eq!(reply["result"]["version"], "1.2.3");
        assert_eq!(reply["result"]["appDataDir"], "data");
    }

    #[tokio::test]
    async fn answers_malformed_json_with_invalid_request_and_a_null_id() {
        let reply = reply("{oops").await;
        assert_eq!(reply["type"], "response");
        assert_eq!(reply["id"], Value::Null);
        assert_eq!(reply["error"]["code"], "INVALID_REQUEST");
    }

    #[tokio::test]
    async fn answers_an_unknown_command_with_unknown_command_and_its_id() {
        let reply = reply(r#"{"type":"request","id":3,"command":{"kind":"explode"}}"#).await;
        assert_eq!(reply["id"], 3);
        assert_eq!(reply["error"]["code"], "UNKNOWN_COMMAND");
        assert_eq!(reply["error"]["details"]["kind"], "explode");
    }

    #[tokio::test]
    async fn use_case_errors_keep_their_code_and_message() {
        let not_open = reply(
            r#"{"type":"request","id":4,"command":{"kind":"undo","projectId":"00000000-0001-7000-8000-000000000000"}}"#,
        )
        .await;
        assert_eq!(not_open["error"]["code"], "PROJECT_NOT_OPEN");
        assert!(
            not_open["error"]["message"]
                .as_str()
                .unwrap()
                .contains("openProject")
        );
        let relative = reply(
            r#"{"type":"request","id":5,"command":{"kind":"openProject","path":"a.seproj"}}"#,
        )
        .await;
        assert_eq!(relative["error"]["code"], "INVALID_REQUEST");
    }

    #[tokio::test]
    async fn every_result_has_the_kind_of_its_command() {
        for kind in [
            "ping",
            "getCoreInfo",
            "getNewProjectDefaults",
            "listRecentProjects",
        ] {
            let request = format!(r#"{{"type":"request","id":1,"command":{{"kind":"{kind}"}}}}"#);
            assert_eq!(reply(&request).await["result"]["kind"], kind);
        }
    }
}

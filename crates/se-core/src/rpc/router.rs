//! `RpcRouter`: one text frame in, one response frame out.

use se_protocol::{Command, ResponseEnvelope, ResponseOutcome, ServerMessage};

use super::{
    encode::encode,
    handlers::{HandlerContext, system},
    request::{Rejection, parse_request},
};

/// Parses requests, dispatches each command to its handler and encodes the response.
///
/// Handlers are synchronous for now. When the first `async` handler arrives, make
/// [`RpcRouter::handle_text`] async and spawn one task per request in `connection.rs`, so a slow
/// command never delays events or other requests.
#[derive(Debug)]
pub struct RpcRouter {
    context: HandlerContext,
}

impl RpcRouter {
    /// A router whose handlers read `context`.
    pub fn new(context: HandlerContext) -> Self {
        Self { context }
    }

    /// Handles one text frame received on `/rpc` and returns the JSON response frame.
    pub fn handle_text(&self, text: &str) -> String {
        let response = match parse_request(text) {
            Ok(request) => {
                tracing::debug!(id = request.id, command = ?request.command, "request");
                ResponseEnvelope {
                    id: Some(request.id),
                    outcome: self.dispatch(&request.command),
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
    fn dispatch(&self, command: &Command) -> ResponseOutcome {
        let result = match command {
            Command::Ping => system::ping(),
            Command::GetCoreInfo => system::get_core_info(&self.context),
        };
        ResponseOutcome::Result(result)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::Value;

    use super::*;
    use crate::app_paths::AppPaths;

    fn router() -> RpcRouter {
        RpcRouter::new(HandlerContext {
            version: "1.2.3",
            paths: AppPaths {
                data_dir: PathBuf::from("data"),
                logs_dir: PathBuf::from("data").join("logs"),
            },
        })
    }

    fn reply(text: &str) -> Value {
        serde_json::from_str(&router().handle_text(text)).unwrap()
    }

    #[test]
    fn answers_ping_with_a_ping_result() {
        assert_eq!(
            router().handle_text(r#"{"type":"request","id":7,"command":{"kind":"ping"}}"#),
            r#"{"type":"response","id":7,"result":{"kind":"ping"}}"#
        );
    }

    #[test]
    fn answers_get_core_info_with_the_context() {
        let reply = reply(r#"{"type":"request","id":8,"command":{"kind":"getCoreInfo"}}"#);
        assert_eq!(reply["id"], 8);
        assert_eq!(reply["result"]["kind"], "getCoreInfo");
        assert_eq!(reply["result"]["version"], "1.2.3");
        assert_eq!(reply["result"]["appDataDir"], "data");
    }

    #[test]
    fn answers_malformed_json_with_invalid_request_and_a_null_id() {
        let reply = reply("{oops");
        assert_eq!(reply["type"], "response");
        assert_eq!(reply["id"], Value::Null);
        assert_eq!(reply["error"]["code"], "INVALID_REQUEST");
    }

    #[test]
    fn answers_an_unknown_command_with_unknown_command_and_its_id() {
        let reply = reply(r#"{"type":"request","id":3,"command":{"kind":"explode"}}"#);
        assert_eq!(reply["id"], 3);
        assert_eq!(reply["error"]["code"], "UNKNOWN_COMMAND");
        assert_eq!(reply["error"]["details"]["kind"], "explode");
    }

    #[test]
    fn every_result_has_the_kind_of_its_command() {
        for kind in ["ping", "getCoreInfo"] {
            let request = format!(r#"{{"type":"request","id":1,"command":{{"kind":"{kind}"}}}}"#);
            assert_eq!(reply(&request)["result"]["kind"], kind);
        }
    }
}

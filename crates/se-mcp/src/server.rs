//! [`McpServer`]: the MCP server handler (`tools/list`, `tools/call`) and [`mcp_service`], the
//! streamable HTTP service se-core mounts at `/mcp` (ADR-0028).
//!
//! Every call is authenticated: the request's `Authorization: Bearer <token>` header must name a
//! running task (se-core's middleware already refused unknown tokens with 401; a run that ended
//! between the two checks gets an `ACCESS_REVOKED` result).

use std::{sync::Arc, time::Instant};

use http::{HeaderMap, header::AUTHORIZATION, request::Parts};
use rmcp::{
    ErrorData, RoleServer, ServerHandler,
    model::{
        CallToolRequestParams, CallToolResponse, Implementation, ListToolsResult,
        PaginatedRequestParams, ServerCapabilities, ServerConfig,
    },
    service::RequestContext,
    transport::streamable_http_server::{
        StreamableHttpServerConfig, StreamableHttpService, session::local::LocalSessionManager,
    },
};
use se_app::AgentContext;

use crate::{
    context::{McpDeps, ToolContext},
    output::{ToolError, codes},
    registry::ToolEntry,
    tools,
};

/// Name of the server, as agents see it (`mcp__simpleeffect__…` in Claude Code).
pub const SERVER_NAME: &str = "simpleeffect";

/// Instructions sent at initialisation; the full guides come with the task's instructions.
const SERVER_INSTRUCTIONS: &str = "Tools of the SimpleEffect video editor for the project of \
    your task. Read the project before acting: get_project_overview, then get_timeline, and \
    get_frames to see the sequence. Change it with the write tools (add_text, update_item, \
    move_item, apply_batch…), inside your task's scope only, and check visual changes with \
    get_frames. Times are seconds or timecodes; outputs give {\"timecode\",\"seconds\"}.";

/// The MCP server: the tool catalogue over the se-app services.
#[derive(Clone)]
pub struct McpServer {
    deps: McpDeps,
    tools: Arc<Vec<ToolEntry>>,
}

impl McpServer {
    /// A server exposing every tool of [`tools::catalogue`].
    #[must_use]
    pub fn new(deps: McpDeps) -> Self {
        Self {
            deps,
            tools: Arc::new(tools::catalogue()),
        }
    }

    /// The tools served, in catalogue order.
    #[must_use]
    pub fn tools(&self) -> &[ToolEntry] {
        &self.tools
    }

    /// Runs tool `name` for `agent` with `arguments` (also used by tests, without HTTP).
    ///
    /// # Errors
    /// [`ErrorData::invalid_params`] for an unknown tool; tool failures are `Ok` results flagged
    /// as errors, so the agent reads them.
    pub async fn call(
        &self,
        agent: AgentContext,
        name: &str,
        arguments: rmcp::model::JsonObject,
    ) -> Result<rmcp::model::CallToolResult, ErrorData> {
        let entry = self
            .tools
            .iter()
            .find(|entry| entry.name() == name)
            .ok_or_else(|| {
                let known: Vec<&str> = self.tools.iter().map(ToolEntry::name).collect();
                ErrorData::invalid_params(
                    format!("unknown tool {name}; the tools are {}", known.join(", ")),
                    None,
                )
            })?;
        let started = Instant::now();
        let task = agent.task_id;
        let context = ToolContext {
            deps: self.deps.clone(),
            agent,
        };
        let outcome = entry.call(context, arguments).await;
        let elapsed_ms = started.elapsed().as_millis();
        Ok(match outcome {
            Ok(output) => {
                tracing::info!(tool = name, %task, elapsed_ms, "MCP tool call");
                output.into_result()
            }
            Err(error) => {
                tracing::info!(tool = name, %task, elapsed_ms, code = %error.code, "MCP tool call failed");
                error.into_result()
            }
        })
    }

    /// The calling run, from the request's bearer token.
    fn caller(&self, context: &RequestContext<RoleServer>) -> Result<AgentContext, ToolError> {
        let token = context
            .extensions
            .get::<Parts>()
            .and_then(|parts| bearer_token(&parts.headers));
        token
            .and_then(|token| self.deps.tokens.resolve(token))
            .ok_or_else(|| {
                ToolError::new(
                    codes::ACCESS_REVOKED,
                    "this task's run is over: its access to the project ended",
                )
            })
    }
}

impl ServerHandler for McpServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(
                Implementation::new(SERVER_NAME, env!("CARGO_PKG_VERSION"))
                    .with_title("SimpleEffect"),
            )
            .with_instructions(SERVER_INSTRUCTIONS)
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl Future<Output = Result<ListToolsResult, ErrorData>> + Send + '_ {
        let definitions = self
            .tools
            .iter()
            .map(|entry| entry.definition.clone())
            .collect();
        std::future::ready(Ok(ListToolsResult::with_all_items(definitions)))
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let agent = match self.caller(&context) {
            Ok(agent) => agent,
            Err(error) => return Ok(error.into_result().into()),
        };
        let arguments = request.arguments.unwrap_or_default();
        Ok(self.call(agent, &request.name, arguments).await?.into())
    }
}

/// The bearer token of `headers` (`Authorization: Bearer <token>`), if any.
#[must_use]
pub fn bearer_token(headers: &HeaderMap) -> Option<&str> {
    let value = headers.get(AUTHORIZATION)?.to_str().ok()?;
    let (scheme, token) = value.split_once(' ')?;
    (scheme.eq_ignore_ascii_case("bearer") && !token.trim().is_empty()).then(|| token.trim())
}

/// The streamable HTTP service se-core mounts at `/mcp`. Sessions are kept in memory (closed
/// after 5 minutes without requests); `Host` must be a loopback name.
#[must_use]
pub fn mcp_service(deps: McpDeps) -> StreamableHttpService<McpServer, LocalSessionManager> {
    let server = McpServer::new(deps);
    StreamableHttpService::new(
        move || Ok(server.clone()),
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    )
}

#[cfg(test)]
mod tests {
    use http::HeaderValue;

    use super::*;

    #[test]
    fn bearer_tokens_are_read_from_the_authorization_header() {
        let mut headers = HeaderMap::new();
        assert_eq!(bearer_token(&headers), None);
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer abc123"));
        assert_eq!(bearer_token(&headers), Some("abc123"));
        headers.insert(AUTHORIZATION, HeaderValue::from_static("bearer  abc123 "));
        assert_eq!(bearer_token(&headers), Some("abc123"));
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Basic abc123"));
        assert_eq!(bearer_token(&headers), None);
        headers.insert(AUTHORIZATION, HeaderValue::from_static("Bearer "));
        assert_eq!(bearer_token(&headers), None);
    }
}

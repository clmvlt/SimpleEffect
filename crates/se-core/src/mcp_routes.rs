//! `/mcp`: the MCP server of AI agents (se-mcp's streamable HTTP service, ADR-0028).
//!
//! Agents are not the app: they never know the session token. This route has its own guard
//! instead of the session-token one: `Host` must be `127.0.0.1:<port>` or `localhost:<port>`
//! (403 otherwise, against DNS rebinding) and `Authorization: Bearer <token>` must name a running
//! task (401 otherwise: unknown or revoked tokens are refused before any MCP message is read).
//! Tokens are never logged.

use std::sync::Arc;

use axum::{
    Router,
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::{self, Next},
    response::{IntoResponse, Response},
};
use se_app::TaskTokenRegistry;
use se_mcp::{McpDeps, bearer_token, mcp_service};

use crate::access_guard::is_loopback_host;

/// What the `/mcp` guard checks requests against.
#[derive(Clone)]
struct McpGuard {
    tokens: Arc<TaskTokenRegistry>,
    port: u16,
}

/// The `/mcp` router for a core listening on `port`.
pub fn routes(deps: McpDeps, port: u16) -> Router {
    let guard = McpGuard {
        tokens: Arc::clone(&deps.tokens),
        port,
    };
    Router::new()
        .nest_service("/mcp", mcp_service(deps))
        .layer(middleware::from_fn_with_state(guard, require_task_token))
}

async fn require_task_token(
    State(guard): State<McpGuard>,
    request: Request,
    next: Next,
) -> Response {
    let host = request
        .headers()
        .get(header::HOST)
        .and_then(|value| value.to_str().ok());
    if !host.is_some_and(|host| is_loopback_host(host, guard.port)) {
        tracing::warn!("MCP request with a foreign Host refused");
        return (
            StatusCode::FORBIDDEN,
            "Forbidden: the Host header must be 127.0.0.1:<port> or localhost:<port>.",
        )
            .into_response();
    }
    let known =
        bearer_token(request.headers()).is_some_and(|token| guard.tokens.resolve(token).is_some());
    if !known {
        tracing::warn!("MCP request without a valid task token refused");
        let mut response = (
            StatusCode::UNAUTHORIZED,
            "Unauthorized: this MCP endpoint serves SimpleEffect's running AI tasks only; the \
             task token is missing, unknown or its run has ended.",
        )
            .into_response();
        response
            .headers_mut()
            .insert(header::WWW_AUTHENTICATE, HeaderValue::from_static("Bearer"));
        return response;
    }
    next.run(request).await
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use axum::{body::Body, http::Request};
    use rmcp::{
        ServiceExt,
        model::{CallToolRequestParams, ContentBlock},
        transport::{
            StreamableHttpClientTransport,
            streamable_http_client::StreamableHttpClientTransportConfig,
        },
    };
    use se_app::{AgentContext, test_support::MediaTestbed};
    use se_domain::TaskId;
    use se_protocol::TaskScope;
    use tower::ServiceExt as _;

    use super::*;

    struct Served {
        bed: MediaTestbed,
        context: AgentContext,
        url: String,
        router: Router,
    }

    /// An open project and `/mcp` served on a random loopback port.
    async fn serve() -> Served {
        let bed = MediaTestbed::new(Path::new("/films"));
        let project_id = bed
            .create_project(Path::new("/films"), "Film")
            .await
            .unwrap();
        let project = bed.projects.project(project_id).await.unwrap();
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let router = routes(
            McpDeps {
                projects: bed.projects.clone(),
                media: bed.media.clone(),
                renders: bed.renders.clone(),
                tokens: bed.tokens.clone(),
                edits: bed.edits.clone(),
                transcription: bed.transcription.clone(),
                smart_cuts: bed.smart_cuts.clone(),
                captions: bed.captions.clone(),
            },
            port,
        );
        let app = router.clone();
        tokio::spawn(async move { axum::serve(listener, app).await });
        Served {
            context: AgentContext {
                project_id,
                task_id: TaskId::from_test_index(9),
                run: 0,
                scope: TaskScope {
                    sequence_id: project.sequences[0].id,
                    range: None,
                    tracks: None,
                },
            },
            url: format!("http://127.0.0.1:{port}/mcp"),
            router,
            bed,
        }
    }

    fn client_config(url: &str, token: &str) -> StreamableHttpClientTransportConfig {
        // reqwest clients need a TLS provider in this workspace, even for `http://`.
        se_transcribe::install_crypto_provider();
        StreamableHttpClientTransportConfig::with_uri(url.to_owned()).auth_header(token.to_owned())
    }

    #[tokio::test]
    async fn a_running_task_lists_tools_reads_the_project_and_sees_frames() {
        let served = serve().await;
        let token = served.bed.tokens.mint(served.context.clone());
        let client = ()
            .serve(StreamableHttpClientTransport::from_config(client_config(
                &served.url,
                token.as_str(),
            )))
            .await
            .expect("a running task's token connects");
        let tools = client.list_all_tools().await.unwrap();
        assert!(tools.iter().any(|tool| tool.name == "get_frames"));
        assert_eq!(tools.len(), 33);

        let mut arguments = rmcp::model::JsonObject::new();
        arguments.insert("text".to_owned(), "Abonne-toi".into());
        arguments.insert("start".to_owned(), 1.into());
        arguments.insert("end".to_owned(), 2.into());
        let added = client
            .call_tool(CallToolRequestParams::new("add_text").with_arguments(arguments))
            .await
            .unwrap();
        assert_ne!(added.is_error, Some(true), "{:?}", added.content);
        let ContentBlock::Text(text) = &added.content[0] else {
            panic!("the write output is text");
        };
        assert!(
            text.text.contains("\"origin\":\"ai_task\""),
            "{}",
            text.text
        );

        let overview = client
            .call_tool(CallToolRequestParams::new("get_project_overview"))
            .await
            .unwrap();
        let ContentBlock::Text(text) = &overview.content[0] else {
            panic!("the overview is text");
        };
        assert!(text.text.contains("\"name\":\"Film\""), "{}", text.text);

        let renderer = served.bed.renderer.clone();
        let renders = served.bed.renders.clone();
        tokio::spawn(async move {
            loop {
                if let Some(request) = renderer.requests().pop() {
                    renders
                        .deliver(&request.request_id, 0, vec![0xFF, 0xD8, 0xFF, 0xE0])
                        .unwrap();
                    break;
                }
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }
        });
        let arguments = serde_json::json!({ "times": [0] });
        let serde_json::Value::Object(arguments) = arguments else {
            unreachable!()
        };
        let frames = client
            .call_tool(CallToolRequestParams::new("get_frames").with_arguments(arguments))
            .await
            .unwrap();
        assert_ne!(frames.is_error, Some(true), "{:?}", frames.content);
        let image = frames
            .content
            .iter()
            .find_map(|block| match block {
                ContentBlock::Image(image) => Some(image),
                _ => None,
            })
            .expect("an image block");
        assert_eq!(image.mime_type, "image/jpeg");
        assert_eq!(image.data, "/9j/4A==");

        served.bed.tokens.revoke(&token);
        let after = client
            .call_tool(CallToolRequestParams::new("get_project_overview"))
            .await;
        assert!(after.is_err(), "a revoked token is refused: {after:?}");
    }

    #[tokio::test]
    async fn unknown_tokens_cannot_connect() {
        let served = serve().await;
        let refused = ()
            .serve(StreamableHttpClientTransport::from_config(client_config(
                &served.url,
                "not-a-task-token",
            )))
            .await;
        assert!(refused.is_err());
    }

    #[tokio::test]
    async fn foreign_hosts_and_missing_tokens_are_refused_before_mcp() {
        let served = serve().await;
        let token = served.bed.tokens.mint(served.context.clone());
        let port = served
            .url
            .split(':')
            .nth(2)
            .unwrap()
            .trim_end_matches("/mcp")
            .to_owned();
        let request = |host: String, authorization: Option<String>| {
            let mut builder = Request::builder()
                .method("POST")
                .uri("/mcp")
                .header(header::HOST, host)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ACCEPT, "application/json, text/event-stream");
            if let Some(authorization) = authorization {
                builder = builder.header(header::AUTHORIZATION, authorization);
            }
            builder.body(Body::from("{}")).unwrap()
        };
        let foreign = served
            .router
            .clone()
            .oneshot(request(
                format!("evil.example:{port}"),
                Some(format!("Bearer {}", token.as_str())),
            ))
            .await
            .unwrap();
        assert_eq!(foreign.status(), StatusCode::FORBIDDEN);
        let missing = served
            .router
            .clone()
            .oneshot(request(format!("127.0.0.1:{port}"), None))
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::UNAUTHORIZED);
        assert_eq!(missing.headers()[header::WWW_AUTHENTICATE], "Bearer");
        let allowed = served
            .router
            .oneshot(request(
                format!("localhost:{port}"),
                Some(format!("Bearer {}", token.as_str())),
            ))
            .await
            .unwrap();
        assert_ne!(allowed.status(), StatusCode::UNAUTHORIZED);
        assert_ne!(allowed.status(), StatusCode::FORBIDDEN);
    }
}

//! `PUT /media/renders/{request_id}/{index}`: the renderer uploads one JPEG per frame an agent
//! asked for (`renderFramesRequested`, ADR-0028); the frame render service answers the agent once
//! every frame arrived. Behind the access guard (`?t=<token>`) like every `/media` route.
//!
//! The renderer's page is another origin than the core (file:// or the dev server), and a `PUT`
//! of `image/jpeg` is not a simple request: the browser asks first (`OPTIONS` preflight), so this
//! route answers CORS for its own method and header only.

use std::sync::Arc;

use axum::{
    Router,
    body::Bytes,
    extract::{DefaultBodyLimit, Path, State},
    http::{Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::put,
};
use se_app::{AppError, FrameRenderService, frame_renders::MAX_RENDERED_FRAME_BYTES};
use tower_http::cors::{AllowOrigin, CorsLayer};

/// The upload router over the frame render service.
pub fn routes(renders: Arc<FrameRenderService>) -> Router {
    Router::new()
        .route("/media/renders/{request_id}/{index}", put(upload))
        .layer(DefaultBodyLimit::max(MAX_RENDERED_FRAME_BYTES + 1))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::PUT])
                .allow_headers([header::CONTENT_TYPE])
                // The session token in the query authenticates; the origin only tells pages
                // apart, and the renderer's origin differs between dev and packaged builds.
                .allow_origin(AllowOrigin::mirror_request()),
        )
        .with_state(renders)
}

async fn upload(
    State(renders): State<Arc<FrameRenderService>>,
    Path((request_id, index)): Path<(String, usize)>,
    body: Bytes,
) -> Response {
    match renders.deliver(&request_id, index, body.to_vec()) {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(error @ AppError::RenderRequestNotFound { .. }) => {
            (StatusCode::NOT_FOUND, error.to_string()).into_response()
        }
        Err(error @ AppError::InvalidRender { .. }) => {
            (StatusCode::BAD_REQUEST, error.to_string()).into_response()
        }
        Err(error) => {
            tracing::error!(%error, "storing a rendered frame failed");
            (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{path::Path as FsPath, sync::Arc};

    use axum::{body::Body, http::Request};
    use se_app::{FrameRenderSpec, test_support::MediaTestbed};
    use se_domain::{Flicks, ProjectId, SequenceId};
    use tower::ServiceExt;

    use super::*;

    fn upload_request(path: &str, body: Vec<u8>) -> Request<Body> {
        Request::builder()
            .method(Method::PUT)
            .uri(path)
            .header(header::CONTENT_TYPE, "image/jpeg")
            .header(header::ORIGIN, "http://localhost:5173")
            .body(Body::from(body))
            .unwrap()
    }

    #[tokio::test]
    async fn uploads_answer_the_waiting_request() {
        let bed = MediaTestbed::new(FsPath::new("/projects"));
        let renders = bed.renders.clone();
        let waiting = tokio::spawn({
            let renders = renders.clone();
            async move {
                renders
                    .render(FrameRenderSpec {
                        project_id: ProjectId::from_test_index(1),
                        sequence_id: SequenceId::from_test_index(1),
                        times: vec![Flicks::ZERO],
                        width: 64,
                        height: 36,
                    })
                    .await
            }
        });
        let request_id = loop {
            if let Some(request) = bed.renderer.requests().pop() {
                break request.request_id;
            }
            tokio::task::yield_now().await;
        };
        let router = routes(renders);
        let refused = router
            .clone()
            .oneshot(upload_request(
                &format!("/media/renders/{request_id}/0"),
                b"png".to_vec(),
            ))
            .await
            .unwrap();
        assert_eq!(refused.status(), StatusCode::BAD_REQUEST);
        let jpeg = vec![0xFF, 0xD8, 0xFF, 0xE0];
        let response = router
            .clone()
            .oneshot(upload_request(
                &format!("/media/renders/{request_id}/0"),
                jpeg.clone(),
            ))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert_eq!(
            response.headers()[header::ACCESS_CONTROL_ALLOW_ORIGIN],
            "http://localhost:5173"
        );
        assert_eq!(waiting.await.unwrap().unwrap(), vec![jpeg.clone()]);
        let late = router
            .oneshot(upload_request(
                &format!("/media/renders/{request_id}/0"),
                jpeg,
            ))
            .await
            .unwrap();
        assert_eq!(late.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn preflights_allow_jpeg_uploads() {
        let bed = MediaTestbed::new(FsPath::new("/projects"));
        let response = routes(Arc::clone(&bed.renders))
            .oneshot(
                Request::builder()
                    .method(Method::OPTIONS)
                    .uri("/media/renders/r/0")
                    .header(header::ORIGIN, "file://")
                    .header(header::ACCESS_CONTROL_REQUEST_METHOD, "PUT")
                    .header(header::ACCESS_CONTROL_REQUEST_HEADERS, "content-type")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(response.status().is_success());
        let methods = response.headers()[header::ACCESS_CONTROL_ALLOW_METHODS]
            .to_str()
            .unwrap();
        assert!(methods.contains("PUT"));
    }
}

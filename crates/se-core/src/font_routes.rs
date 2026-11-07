//! `GET /media/fonts/{id}`: the file of a font face of the catalogue (`listFonts`), loaded by the
//! renderer with `FontFace` (prompt 15). Behind the access guard like every route (`?t=<token>`).
//! Bundled faces come from memory, installed ones from their file (tower-http's `ServeFile`:
//! Range, `HEAD`, conditional requests). 404 for unknown ids.

use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::{Path, Request, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use se_app::ports::{FontCatalog, FontData};
use tower::ServiceExt;
use tower_http::services::ServeFile;

/// The `/media/fonts` router over the font catalogue.
pub fn routes(fonts: Arc<dyn FontCatalog>) -> Router {
    Router::new()
        .route("/media/fonts/{id}", get(font_face))
        .with_state(fonts)
}

async fn font_face(
    State(fonts): State<Arc<dyn FontCatalog>>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    let Some(data) = fonts.face_data(&id).await else {
        return (
            StatusCode::NOT_FOUND,
            format!("no font face '{id}': list the fonts again (listFonts)"),
        )
            .into_response();
    };
    let (mut response, content_type) = match data {
        FontData::Embedded {
            bytes,
            content_type,
        } => (Body::from(bytes).into_response(), content_type),
        FontData::File { path, content_type } => {
            match ServeFile::new(&path).oneshot(request).await {
                Ok(response) => (response.map(Body::new), content_type),
                Err(error) => {
                    tracing::warn!(%error, "serving a font file failed");
                    return (StatusCode::INTERNAL_SERVER_ERROR, "reading the font failed")
                        .into_response();
                }
            }
        }
    };
    let headers = response.headers_mut();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    // An installed font may be updated in place: always revalidate.
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    response
}

#[cfg(test)]
mod tests {
    use axum::http::Request as HttpRequest;
    use se_app::test_support::{FAKE_FONT_BYTES, FakeFontCatalog};

    use super::*;

    async fn get(uri: &str) -> Response {
        routes(Arc::new(FakeFontCatalog))
            .oneshot(HttpRequest::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn serves_known_faces_with_their_type() {
        let response = get("/media/fonts/inter-latin").await;
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "font/woff2");
        let body = axum::body::to_bytes(response.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(&body[..], FAKE_FONT_BYTES);
    }

    #[tokio::test]
    async fn unknown_faces_are_404() {
        assert_eq!(
            get("/media/fonts/nope").await.status(),
            StatusCode::NOT_FOUND
        );
    }
}

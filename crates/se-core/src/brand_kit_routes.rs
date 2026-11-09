//! `GET /media/brand-kits/{kit}/logos/{file}`: a logo of a library brand kit (prompt 25), shown
//! by the brand kit editor. Behind the access guard like every route (`?t=<token>`); tower-http's
//! `ServeFile` guesses the image type from the extension. Only plain logo file names of a kit
//! are served (400 otherwise), 404 when the file does not exist.

use std::sync::Arc;

use axum::{
    Router,
    body::Body,
    extract::{Path, Request, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use se_app::{brand_kits::is_logo_file_name, ports::BrandKitLibrary};
use se_domain::BrandKitId;
use tower::ServiceExt;
use tower_http::services::ServeFile;

use crate::media_cors::read_only_cors;

/// The `/media/brand-kits` router over the kits on disk.
pub fn routes(kits: Arc<dyn BrandKitLibrary>) -> Router {
    Router::new()
        .route("/media/brand-kits/{kit}/logos/{file}", get(logo))
        .layer(read_only_cors())
        .with_state(kits)
}

async fn logo(
    State(kits): State<Arc<dyn BrandKitLibrary>>,
    Path((kit, file)): Path<(String, String)>,
    request: Request,
) -> Response {
    let name = format!("logos/{file}");
    let Ok(kit) = BrandKitId::parse(&kit) else {
        return (StatusCode::BAD_REQUEST, "malformed brand kit id").into_response();
    };
    if !is_logo_file_name(&name) {
        return (StatusCode::BAD_REQUEST, "not a logo file name").into_response();
    }
    let path = kits.logo_path(kit, &name);
    if !path.is_file() {
        return (
            StatusCode::NOT_FOUND,
            "no such logo: list the brand kits again (listBrandKits)",
        )
            .into_response();
    }
    match ServeFile::new(&path).oneshot(request).await {
        Ok(response) => {
            let mut response = response.map(Body::new);
            response
                .headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
            response
        }
        Err(error) => {
            tracing::warn!(%error, "serving a brand kit logo failed");
            (StatusCode::INTERNAL_SERVER_ERROR, "reading the logo failed").into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use axum::http::Request as HttpRequest;
    use se_store::FsBrandKitLibrary;

    use super::*;

    async fn get(kits: Arc<dyn BrandKitLibrary>, uri: &str) -> Response {
        routes(kits)
            .oneshot(HttpRequest::get(uri).body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn serves_logo_files_and_nothing_else() {
        let root = tempfile::tempdir().unwrap();
        let kit = BrandKitId::from_test_index(1);
        let folder = root.path().join(kit.to_string()).join("logos");
        fs::create_dir_all(&folder).unwrap();
        fs::write(folder.join("a.png"), b"PNG").unwrap();
        fs::write(
            root.path().join(kit.to_string()).join("brand-kit.json"),
            b"{}",
        )
        .unwrap();
        let kits: Arc<dyn BrandKitLibrary> =
            Arc::new(FsBrandKitLibrary::new(root.path().to_path_buf()));
        let base = format!("/media/brand-kits/{kit}/logos");
        let found = get(kits.clone(), &format!("{base}/a.png")).await;
        assert_eq!(found.status(), StatusCode::OK);
        assert_eq!(found.headers()[header::CONTENT_TYPE], "image/png");
        assert_eq!(
            get(kits.clone(), &format!("{base}/b.png")).await.status(),
            StatusCode::NOT_FOUND
        );
        assert_eq!(
            get(kits.clone(), &format!("{base}/..%2Fbrand-kit.json"))
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
        assert_eq!(
            get(kits, "/media/brand-kits/nope/logos/a.png")
                .await
                .status(),
            StatusCode::BAD_REQUEST
        );
    }
}

//! `/media` routes: an asset's original and its derived files, with Range requests (206/416),
//! conditional requests and correct content types (`docs/architecture.md` §7). Behind the access
//! guard like every route: `?t=<token>` is required (media elements cannot send headers).
//!
//! | Route | File |
//! |---|---|
//! | `GET /media/assets/{id}/original` | the user's file (type guessed from its extension) |
//! | `GET /media/assets/{id}/thumbnails` | the sheet layout (`ThumbnailSheet` JSON) |
//! | `GET /media/assets/{id}/thumbnails/sheet` | the thumbnail sheet (JPEG) |
//! | `GET /media/assets/{id}/thumbnails/poster` | the poster (JPEG) |
//! | `GET /media/assets/{id}/peaks/{level}` | `SEPK` peaks, `level` = samples per peak (256, 1024, 4096) |
//! | `GET /media/assets/{id}/mezzanine` | the mezzanine, once ready (type from its extension) |
//! | `GET /media/assets/{id}/proxy` | the proxy, once ready |
//!
//! 404 when no open project has the asset, or the file does not exist (yet).

use std::{path::PathBuf, sync::Arc};

use axum::{
    Router,
    body::Body,
    extract::{Path, Request, State},
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use se_app::MediaService;
use se_domain::AssetId;
use se_media::{
    cache_layout::{THUMBNAIL_INFO, THUMBNAIL_POSTER, THUMBNAIL_SHEET, peaks_file},
    peaks::PEAK_LEVELS,
};
use tower::ServiceExt;
use tower_http::services::ServeFile;

/// The `/media` router over the media use cases.
pub fn routes(media: Arc<MediaService>) -> Router {
    Router::new()
        .route("/media/assets/{id}/original", get(original))
        .route("/media/assets/{id}/thumbnails", get(thumbnail_info))
        .route("/media/assets/{id}/thumbnails/sheet", get(thumbnail_sheet))
        .route(
            "/media/assets/{id}/thumbnails/poster",
            get(thumbnail_poster),
        )
        .route("/media/assets/{id}/peaks/{level}", get(peaks))
        .route("/media/assets/{id}/mezzanine", get(mezzanine))
        .route("/media/assets/{id}/proxy", get(proxy))
        .with_state(media)
}

/// What to serve, before the asset is looked up.
enum Wanted {
    Original,
    Mezzanine,
    Proxy,
    Derived {
        file: String,
        content_type: &'static str,
    },
}

async fn original(
    State(media): State<Arc<MediaService>>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    serve(&media, &id, Wanted::Original, request).await
}

async fn mezzanine(
    State(media): State<Arc<MediaService>>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    serve(&media, &id, Wanted::Mezzanine, request).await
}

async fn proxy(
    State(media): State<Arc<MediaService>>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    serve(&media, &id, Wanted::Proxy, request).await
}

async fn thumbnail_info(
    State(media): State<Arc<MediaService>>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    let wanted = Wanted::Derived {
        file: THUMBNAIL_INFO.to_owned(),
        content_type: "application/json",
    };
    serve(&media, &id, wanted, request).await
}

async fn thumbnail_sheet(
    State(media): State<Arc<MediaService>>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    let wanted = Wanted::Derived {
        file: THUMBNAIL_SHEET.to_owned(),
        content_type: "image/jpeg",
    };
    serve(&media, &id, wanted, request).await
}

async fn thumbnail_poster(
    State(media): State<Arc<MediaService>>,
    Path(id): Path<String>,
    request: Request,
) -> Response {
    let wanted = Wanted::Derived {
        file: THUMBNAIL_POSTER.to_owned(),
        content_type: "image/jpeg",
    };
    serve(&media, &id, wanted, request).await
}

async fn peaks(
    State(media): State<Arc<MediaService>>,
    Path((id, level)): Path<(String, String)>,
    request: Request,
) -> Response {
    let Some(level) = level
        .parse::<u32>()
        .ok()
        .filter(|level| PEAK_LEVELS.contains(level))
    else {
        let levels = PEAK_LEVELS.map(|level| level.to_string()).join(", ");
        return not_found(&format!("unknown peak level: use one of {levels}"));
    };
    let wanted = Wanted::Derived {
        file: peaks_file(level),
        content_type: "application/octet-stream",
    };
    serve(&media, &id, wanted, request).await
}

/// Finds the file and hands the request to tower-http's file service (Range, `HEAD`,
/// `If-Modified-Since`).
async fn serve(media: &MediaService, id: &str, wanted: Wanted, request: Request) -> Response {
    let Ok(asset_id) = AssetId::parse(id) else {
        return (
            StatusCode::BAD_REQUEST,
            format!("'{id}' is not an asset id"),
        )
            .into_response();
    };
    let Some(files) = media.asset_files(asset_id).await else {
        return not_found(&format!("no open project has asset {asset_id}"));
    };
    let (path, content_type): (PathBuf, Option<&'static str>) = match wanted {
        Wanted::Original => (files.original, None),
        Wanted::Mezzanine => match files.mezzanine {
            Some(path) => (path, None),
            None => return not_found("this asset has no ready mezzanine"),
        },
        Wanted::Proxy => match files.proxy {
            Some(path) => (path, None),
            None => return not_found("this asset has no ready proxy"),
        },
        Wanted::Derived { file, content_type } => match files.cache_dir {
            Some(dir) => (dir.join(file), Some(content_type)),
            None => return not_found("this asset has no derived files"),
        },
    };
    if !tokio::fs::metadata(&path)
        .await
        .is_ok_and(|metadata| metadata.is_file())
    {
        return not_found("the file does not exist (missing original, or not generated yet)");
    }
    let mut response = match ServeFile::new(&path).oneshot(request).await {
        Ok(response) => response.map(Body::new),
        Err(error) => {
            tracing::warn!(%error, "serving a media file failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, "reading the file failed").into_response();
        }
    };
    let headers = response.headers_mut();
    if let Some(content_type) = content_type {
        headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    }
    // Derived files change after a relink; originals may be replaced: always revalidate.
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    response
}

fn not_found(message: &str) -> Response {
    (StatusCode::NOT_FOUND, message.to_owned()).into_response()
}

#[cfg(test)]
mod tests {
    use std::path::Path as FsPath;

    use axum::http::{Request as HttpRequest, header::RANGE};
    use se_app::test_support::{MediaTestbed, probed_video};

    use super::*;

    /// A media router whose open project has one asset pointing at a real 1000-byte file.
    async fn router_with_asset(dir: &FsPath) -> (Router, AssetId, MediaTestbed) {
        let original = dir.join("clip.mp4");
        std::fs::write(
            &original,
            (0..1000_u32).map(|i| (i % 251) as u8).collect::<Vec<_>>(),
        )
        .unwrap();
        let testbed = MediaTestbed::new(dir);
        testbed.files.add(&original, 1, probed_video());
        let project_id = testbed.create_project(dir, "Film").await.unwrap();
        let report = testbed
            .media
            .import_files(project_id, &[original])
            .await
            .unwrap();
        (routes(testbed.media.clone()), report.imported[0], testbed)
    }

    async fn get(router: &Router, uri: &str, range: Option<&str>) -> Response {
        let mut request = HttpRequest::get(uri);
        if let Some(range) = range {
            request = request.header(RANGE, range);
        }
        router
            .clone()
            .oneshot(request.body(Body::empty()).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn originals_honour_range_requests() {
        let dir = tempfile::tempdir().unwrap();
        let (router, asset, _projects) = router_with_asset(dir.path()).await;
        let uri = format!("/media/assets/{asset}/original");

        let whole = get(&router, &uri, None).await;
        assert_eq!(whole.status(), StatusCode::OK);
        assert_eq!(whole.headers()[header::CONTENT_TYPE], "video/mp4");
        assert_eq!(whole.headers()[header::ACCEPT_RANGES], "bytes");

        let part = get(&router, &uri, Some("bytes=10-19")).await;
        assert_eq!(part.status(), StatusCode::PARTIAL_CONTENT);
        assert_eq!(part.headers()[header::CONTENT_RANGE], "bytes 10-19/1000");
        let body = axum::body::to_bytes(part.into_body(), 100).await.unwrap();
        assert_eq!(body.len(), 10);
        assert_eq!(body[0], 10);

        let beyond = get(&router, &uri, Some("bytes=5000-6000")).await;
        assert_eq!(beyond.status(), StatusCode::RANGE_NOT_SATISFIABLE);
    }

    #[tokio::test]
    async fn unknown_assets_missing_outputs_and_bad_levels_are_404() {
        let dir = tempfile::tempdir().unwrap();
        let (router, asset, _projects) = router_with_asset(dir.path()).await;
        let other = AssetId::from_test_index(99);
        let unknown = get(&router, &format!("/media/assets/{other}/original"), None).await;
        assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
        let not_ready = get(&router, &format!("/media/assets/{asset}/thumbnails"), None).await;
        assert_eq!(not_ready.status(), StatusCode::NOT_FOUND);
        let level = get(&router, &format!("/media/assets/{asset}/peaks/512"), None).await;
        assert_eq!(level.status(), StatusCode::NOT_FOUND);
        let mezzanine = get(&router, &format!("/media/assets/{asset}/mezzanine"), None).await;
        assert_eq!(mezzanine.status(), StatusCode::NOT_FOUND);
        let proxy = get(&router, &format!("/media/assets/{asset}/proxy"), None).await;
        assert_eq!(proxy.status(), StatusCode::NOT_FOUND);
        let bad_id = get(&router, "/media/assets/nope/original", None).await;
        assert_eq!(bad_id.status(), StatusCode::BAD_REQUEST);
    }
}

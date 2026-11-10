//! `/media/exports/{id}/…` (prompt 27, ADR-0038): what the renderer uploads for an export it
//! renders, the export's picture for the list, and the media of its snapshot. Behind the access
//! guard (`?t=<token>`) like every `/media` route.
//!
//! | Route | Role |
//! |---|---|
//! | `PUT …/data?position=N` | bytes of the muxed file (or of the picture before ffmpeg adds AAC) |
//! | `PUT …/audio?position=N` | bytes of the WAV ffmpeg reads |
//! | `PUT …/frames` | the next RGBA frame of a master (ProRes, DNxHR) |
//! | `PUT …/thumbnail`, `GET …/thumbnail` | a JPEG of the export for the list |
//! | `GET …/assets/{asset}/original`, `…/mezzanine` | files of the export's snapshot (Range) |
//!
//! Each upload answers once written (204), so the renderer's muxer feels the disk's pace. 404 for
//! an unknown export, 409 when it no longer renders (cancelled, failed), 400 for refused
//! content, 500 when writing failed (the export fails with the reason). CORS for `PUT` and reads:
//! the renderer's page is another origin.

use std::{collections::HashMap, path::PathBuf};

use axum::{
    Router,
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, Path, Query, Request, State},
    http::{HeaderValue, Method, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, put},
};
use se_app::{
    AppError, ExportService,
    exports::{ExportMediaVariant, settings::MAX_FRAME_BYTES},
    ports::ExportStream,
};
use se_domain::{AssetId, ExportId};
use tower::ServiceExt;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    services::ServeFile,
};

/// The export router over the export service.
pub fn routes(exports: ExportService) -> Router {
    Router::new()
        .route("/media/exports/{id}/data", put(upload_data))
        .route("/media/exports/{id}/audio", put(upload_audio))
        .route("/media/exports/{id}/frames", put(upload_frame))
        .route(
            "/media/exports/{id}/thumbnail",
            put(upload_thumbnail).get(thumbnail),
        )
        .route("/media/exports/{id}/assets/{asset}/{variant}", get(media))
        // The largest upload is an 8K RGBA frame; chunks of muxed files are far smaller.
        .layer(DefaultBodyLimit::max(MAX_FRAME_BYTES + 1))
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::HEAD, Method::PUT])
                .allow_headers([header::CONTENT_TYPE, header::RANGE])
                .expose_headers([
                    header::CONTENT_RANGE,
                    header::CONTENT_LENGTH,
                    header::ACCEPT_RANGES,
                    header::ETAG,
                ])
                // The session token in the query authenticates; the renderer's origin differs
                // between dev and packaged builds.
                .allow_origin(AllowOrigin::mirror_request()),
        )
        .with_state(exports)
}

/// The query of a stream upload (`?position=N`, next to the session token).
type UploadQuery = Query<HashMap<String, String>>;

/// Where an upload goes in its stream.
fn position_of(query: &HashMap<String, String>) -> Result<u64, Refusal> {
    query
        .get("position")
        .and_then(|text| text.parse().ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                "an upload needs ?position=<byte offset>".to_owned(),
            )
        })
}

/// A request refused before reaching the export service.
type Refusal = (StatusCode, String);

fn export_id(text: &str) -> Result<ExportId, Refusal> {
    ExportId::parse(text).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            format!("'{text}' is not an export id"),
        )
    })
}

async fn upload_data(
    State(exports): State<ExportService>,
    Path(id): Path<String>,
    Query(query): UploadQuery,
    body: Bytes,
) -> Response {
    upload_stream(&exports, &id, ExportStream::Data, &query, body).await
}

async fn upload_audio(
    State(exports): State<ExportService>,
    Path(id): Path<String>,
    Query(query): UploadQuery,
    body: Bytes,
) -> Response {
    upload_stream(&exports, &id, ExportStream::Audio, &query, body).await
}

async fn upload_stream(
    exports: &ExportService,
    id: &str,
    stream: ExportStream,
    query: &HashMap<String, String>,
    body: Bytes,
) -> Response {
    let (export_id, position) = match (export_id(id), position_of(query)) {
        (Ok(export_id), Ok(position)) => (export_id, position),
        (Err(refusal), _) | (_, Err(refusal)) => return refusal.into_response(),
    };
    answer(
        exports
            .write(export_id, stream, position, Vec::from(body))
            .await,
    )
}

async fn upload_frame(
    State(exports): State<ExportService>,
    Path(id): Path<String>,
    body: Bytes,
) -> Response {
    let export_id = match export_id(&id) {
        Ok(export_id) => export_id,
        Err(refusal) => return refusal.into_response(),
    };
    answer(exports.write_frame(export_id, Vec::from(body)).await)
}

async fn upload_thumbnail(
    State(exports): State<ExportService>,
    Path(id): Path<String>,
    body: Bytes,
) -> Response {
    let export_id = match export_id(&id) {
        Ok(export_id) => export_id,
        Err(refusal) => return refusal.into_response(),
    };
    answer(exports.set_thumbnail(export_id, Vec::from(body)))
}

/// The status of an upload's outcome.
fn answer(outcome: Result<(), AppError>) -> Response {
    let status = match &outcome {
        Ok(()) => return StatusCode::NO_CONTENT.into_response(),
        Err(AppError::ExportNotFound { .. }) => StatusCode::NOT_FOUND,
        Err(AppError::ExportNotRendering { .. }) => StatusCode::CONFLICT,
        Err(AppError::InvalidExport { .. }) => StatusCode::BAD_REQUEST,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };
    let message = outcome
        .err()
        .map(|error| error.to_string())
        .unwrap_or_default();
    (status, message).into_response()
}

async fn thumbnail(State(exports): State<ExportService>, Path(id): Path<String>) -> Response {
    let export_id = match export_id(&id) {
        Ok(export_id) => export_id,
        Err(refusal) => return refusal.into_response(),
    };
    let Some(jpeg) = exports.thumbnail(export_id) else {
        return (StatusCode::NOT_FOUND, "this export has no picture").into_response();
    };
    (
        [
            (header::CONTENT_TYPE, HeaderValue::from_static("image/jpeg")),
            (header::CACHE_CONTROL, HeaderValue::from_static("no-cache")),
        ],
        jpeg.as_ref().clone(),
    )
        .into_response()
}

async fn media(
    State(exports): State<ExportService>,
    Path((id, asset, variant)): Path<(String, String, String)>,
    request: Request,
) -> Response {
    let export_id = match export_id(&id) {
        Ok(export_id) => export_id,
        Err(refusal) => return refusal.into_response(),
    };
    let Ok(asset_id) = AssetId::parse(&asset) else {
        return (
            StatusCode::BAD_REQUEST,
            format!("'{asset}' is not an asset id"),
        )
            .into_response();
    };
    let variant = match variant.as_str() {
        "original" => ExportMediaVariant::Original,
        "mezzanine" => ExportMediaVariant::Mezzanine,
        _ => {
            return (StatusCode::NOT_FOUND, "use original or mezzanine").into_response();
        }
    };
    match exports.media_file(export_id, asset_id, variant) {
        Some(path) => serve_file(path, request).await,
        None => (
            StatusCode::NOT_FOUND,
            "no running export reads this file (finished export, unknown asset or no mezzanine)",
        )
            .into_response(),
    }
}

/// Serves a file with Range support, revalidated on each read.
async fn serve_file(path: PathBuf, request: Request) -> Response {
    if !tokio::fs::metadata(&path)
        .await
        .is_ok_and(|metadata| metadata.is_file())
    {
        return (StatusCode::NOT_FOUND, "the file does not exist any more").into_response();
    }
    let mut response = match ServeFile::new(&path).oneshot(request).await {
        Ok(response) => response.map(Body::new),
        Err(error) => {
            tracing::warn!(%error, "serving an export's media failed");
            return (StatusCode::INTERNAL_SERVER_ERROR, "reading the file failed").into_response();
        }
    };
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-cache"));
    response
}

#[cfg(test)]
mod tests {
    use std::path::{Path as FsPath, PathBuf as FsPathBuf};

    use se_app::test_support::{MediaTestbed, probed_video, settle};
    use se_domain::test_support::example_project;
    use se_protocol::{
        AudioEncoderChoice, AudioExportCodec, AudioExportSettings, ExportContainer, ExportSettings,
        StartExportRequest,
    };

    use super::*;

    fn request(method: Method, uri: &str, body: Vec<u8>) -> Request<Body> {
        Request::builder()
            .method(method)
            .uri(uri)
            .header(header::ORIGIN, "http://localhost:5173")
            .body(Body::from(body))
            .unwrap()
    }

    async fn rendering_export(bed: &MediaTestbed, dir: &FsPath) -> ExportId {
        let mut project = example_project();
        let original = dir.join("clip.mp4");
        std::fs::write(&original, b"0123456789").unwrap();
        project.assets[0].original_path = original.to_string_lossy().into_owned();
        for (content, asset) in (1..).zip(&project.assets) {
            bed.files
                .add(asset.original_path.clone(), content, probed_video());
        }
        bed.repository.insert(
            FsPathBuf::from("/films/A.seproj"),
            project.clone(),
            Vec::new(),
        );
        let session = bed
            .projects
            .open_project(FsPath::new("/films/A.seproj"))
            .await
            .unwrap();
        let export = bed
            .exports
            .start(StartExportRequest {
                project_id: session.snapshot.project.id,
                sequence_id: project.sequences[0].id,
                settings: ExportSettings {
                    container: ExportContainer::Wav,
                    video: None,
                    audio: Some(AudioExportSettings {
                        codec: AudioExportCodec::Pcm,
                        bits_per_second: 1_536_000,
                        encoder: AudioEncoderChoice::WebCodecs,
                    }),
                },
                range: None,
                path: dir.join("son.wav").to_string_lossy().into_owned(),
                preset_name: "WAV".to_owned(),
            })
            .await
            .unwrap();
        settle().await;
        export.id
    }

    #[tokio::test]
    async fn uploads_are_written_in_place_and_refused_once_finished() {
        let dir = tempfile::tempdir().unwrap();
        let bed = MediaTestbed::new(FsPath::new("/films"));
        let export_id = rendering_export(&bed, dir.path()).await;
        let router = routes(bed.exports.clone());
        let base = format!("/media/exports/{export_id}");
        let written = router
            .clone()
            .oneshot(request(
                Method::PUT,
                &format!("{base}/data?position=2"),
                b"cd".to_vec(),
            ))
            .await
            .unwrap();
        assert_eq!(written.status(), StatusCode::NO_CONTENT);
        assert!(
            written
                .headers()
                .contains_key(header::ACCESS_CONTROL_ALLOW_ORIGIN)
        );
        let no_position = router
            .clone()
            .oneshot(request(
                Method::PUT,
                &format!("{base}/data"),
                b"ab".to_vec(),
            ))
            .await
            .unwrap();
        assert_eq!(no_position.status(), StatusCode::BAD_REQUEST);
        let not_jpeg = router
            .clone()
            .oneshot(request(
                Method::PUT,
                &format!("{base}/thumbnail"),
                b"png".to_vec(),
            ))
            .await
            .unwrap();
        assert_eq!(not_jpeg.status(), StatusCode::BAD_REQUEST);
        let jpeg = router
            .clone()
            .oneshot(request(
                Method::PUT,
                &format!("{base}/thumbnail"),
                vec![0xFF, 0xD8, 0xFF, 0xE0],
            ))
            .await
            .unwrap();
        assert_eq!(jpeg.status(), StatusCode::NO_CONTENT);
        let picture = router
            .clone()
            .oneshot(request(
                Method::GET,
                &format!("{base}/thumbnail"),
                Vec::new(),
            ))
            .await
            .unwrap();
        assert_eq!(picture.headers()[header::CONTENT_TYPE], "image/jpeg");

        bed.exports.cancel(export_id).unwrap();
        settle().await;
        let late = router
            .clone()
            .oneshot(request(
                Method::PUT,
                &format!("{base}/audio?position=0"),
                b"x".to_vec(),
            ))
            .await
            .unwrap();
        assert_eq!(late.status(), StatusCode::CONFLICT);
        let unknown = router
            .oneshot(request(
                Method::PUT,
                "/media/exports/00000000-0077-7000-8000-000000000000/frames",
                Vec::new(),
            ))
            .await
            .unwrap();
        assert_eq!(unknown.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn the_snapshot_media_are_served_while_the_export_runs() {
        let dir = tempfile::tempdir().unwrap();
        let bed = MediaTestbed::new(FsPath::new("/films"));
        let export_id = rendering_export(&bed, dir.path()).await;
        let asset = example_project().assets[0].id;
        let router = routes(bed.exports.clone());
        let missing = router
            .clone()
            .oneshot(request(
                Method::GET,
                &format!("/media/exports/{export_id}/assets/{asset}/proxy"),
                Vec::new(),
            ))
            .await
            .unwrap();
        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
        let mut ranged = request(
            Method::GET,
            &format!("/media/exports/{export_id}/assets/{asset}/original"),
            Vec::new(),
        );
        ranged
            .headers_mut()
            .insert(header::RANGE, HeaderValue::from_static("bytes=2-5"));
        let partial = router.clone().oneshot(ranged).await.unwrap();
        assert_eq!(partial.status(), StatusCode::PARTIAL_CONTENT);
        let text = axum::body::to_bytes(partial.into_body(), 1024)
            .await
            .unwrap();
        assert_eq!(&text[..], b"2345");

        bed.exports.cancel(export_id).unwrap();
        settle().await;
        let finished = router
            .oneshot(request(
                Method::GET,
                &format!("/media/exports/{export_id}/assets/{asset}/original"),
                Vec::new(),
            ))
            .await
            .unwrap();
        assert_eq!(finished.status(), StatusCode::NOT_FOUND);
    }
}

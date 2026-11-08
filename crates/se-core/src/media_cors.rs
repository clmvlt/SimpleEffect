//! CORS for the read-only `/media` routes (files of assets, thumbnails, peaks, fonts).
//!
//! The renderer's page is another origin than the core: `file://` in packaged builds, the Vite
//! dev server (`http://localhost:<port>`) in development. `<img>` and `<video>` elements need no
//! CORS, but the engine reads media with `fetch` (Mediabunny's Range requests for decoding,
//! sheets and peaks as JSON/binary, fonts through `FontFace`): without these headers every read
//! failed in development and the viewer reported « média n'a pas pu être décodé ».

use axum::http::{Method, header};
use tower_http::cors::{AllowOrigin, CorsLayer};

/// The CORS layer of the read-only media routes: `GET`/`HEAD`, the `Range` request header, and the
/// response headers Mediabunny reads to page through a file.
pub fn read_only_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_methods([Method::GET, Method::HEAD])
        .allow_headers([header::RANGE])
        .expose_headers([
            header::CONTENT_RANGE,
            header::CONTENT_LENGTH,
            header::ACCEPT_RANGES,
            header::ETAG,
        ])
        // The session token in the query authenticates (like the frame uploads); the origin only
        // tells pages apart, and the renderer's origin differs between dev and packaged builds.
        .allow_origin(AllowOrigin::mirror_request())
}

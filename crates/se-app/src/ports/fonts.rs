//! Font port: the fonts texts may use, bundled with the app or installed on the system
//! (implemented by se-fonts, served by se-core under `/media/fonts/{id}`).

use std::path::PathBuf;

use async_trait::async_trait;
use se_protocol::FontFamily;

/// Where the bytes of a font face are.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FontData {
    /// A file on disk (system fonts).
    File {
        /// The font file.
        path: PathBuf,
        /// Its MIME type (`font/ttf`, `font/otf`, `font/collection`).
        content_type: &'static str,
    },
    /// Bytes embedded in the binary (bundled fonts).
    Embedded {
        /// The font file's bytes.
        bytes: &'static [u8],
        /// Its MIME type (`font/woff2`).
        content_type: &'static str,
    },
}

/// The font catalogue. Listing system fonts reads every installed font once (it may take a
/// moment on the first call); later calls reuse the result.
#[async_trait]
pub trait FontCatalog: Send + Sync {
    /// Every family: bundled ones first, then installed ones, each sorted by name.
    async fn families(&self) -> Vec<FontFamily>;

    /// The bytes of the face with this id (`FontFace::id`), or `None` when unknown.
    async fn face_data(&self, id: &str) -> Option<FontData>;
}

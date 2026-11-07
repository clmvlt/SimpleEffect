//! [`FakeFontCatalog`]: a fixed font catalogue for the tests of the font routes and commands.

use async_trait::async_trait;
use se_protocol::{FontFace, FontFamily, FontSource};

use crate::ports::{FontCatalog, FontData};

/// Bytes served for the fake face `inter-latin`.
pub const FAKE_FONT_BYTES: &[u8] = b"wOF2-fake-font";

/// A catalogue with one bundled family, `Inter`, whose face `inter-latin` is embedded.
#[derive(Debug, Default)]
pub struct FakeFontCatalog;

#[async_trait]
impl FontCatalog for FakeFontCatalog {
    async fn families(&self) -> Vec<FontFamily> {
        vec![FontFamily {
            name: "Inter".to_owned(),
            source: FontSource::Bundled,
            faces: vec![FontFace {
                id: "inter-latin".to_owned(),
                weight_min: 100,
                weight_max: 900,
                italic: false,
                unicode_range: None,
                format: "woff2".to_owned(),
            }],
        }]
    }

    async fn face_data(&self, id: &str) -> Option<FontData> {
        (id == "inter-latin").then_some(FontData::Embedded {
            bytes: FAKE_FONT_BYTES,
            content_type: "font/woff2",
        })
    }
}

//! DTOs of the font catalogue (`listFonts`, prompt 15): the fonts texts may use, bundled with the
//! app or installed on the system, each face served by `GET /media/fonts/{id}`.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A font family and its faces.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FontFamily {
    /// Family name, as texts name it (`fontFamily`).
    pub name: String,
    /// Where it comes from.
    pub source: FontSource,
    /// Its faces (weights and styles), lightest first.
    pub faces: Vec<FontFace>,
}

/// Where a font family comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum FontSource {
    /// Shipped with SimpleEffect: available on every machine.
    Bundled,
    /// Installed on this machine: a project using it may look different elsewhere.
    System,
}

/// One face of a family: a file (or part of one) covering some weights and a style.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct FontFace {
    /// Id of the face, for `GET /media/fonts/{id}`.
    pub id: String,
    /// Lightest weight it draws (100 to 900; variable fonts cover a range).
    pub weight_min: u16,
    /// Heaviest weight it draws.
    pub weight_max: u16,
    /// Italic or oblique.
    pub italic: bool,
    /// The CSS `unicode-range` it covers (`U+0000-00FF, U+0131`), when it is a subset.
    pub unicode_range: Option<String>,
    /// File format for CSS (`woff2`, `truetype`, `opentype`, `collection`).
    pub format: String,
}

//! [`BrandKit`]: the visual identity reused by the user and agents.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::color::Color;

/// Colours and fonts of the user's brand.
///
/// TODO(prompt-25): logo asset, default caption style, named colour roles and the global library.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct BrandKit {
    /// Brand colours, most important first.
    pub colors: Vec<Color>,
    /// Font family names, most important first.
    pub fonts: Vec<String>,
}

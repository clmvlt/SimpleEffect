//! [`FadeShape`]: how the level of a fade moves between silence (or black) and full.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// The curve of a clip fade, applied to its sound gain and to its picture opacity.
///
/// The curves themselves live in `@se/engine` (`fadeGain`), the single evaluator shared by
/// preview and export; the domain only names them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum FadeShape {
    /// The level changes at a constant rate (gain = progress).
    #[default]
    Linear,
    /// The level follows a quarter sine (gain = sin(progress × π/2)): no dip in loudness when a
    /// fade-out overlaps a fade-in, the usual choice for sound.
    ConstantPower,
    /// The level changes evenly in decibels, from −60 dB to full: a fade-in stays quiet longer,
    /// a fade-out drops quickly at first.
    Logarithmic,
}

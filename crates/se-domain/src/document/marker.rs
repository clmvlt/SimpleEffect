//! [`Marker`]: a labelled point on a sequence's timeline.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::{ids::MarkerId, time::Flicks};

/// A labelled instant of a sequence (a beat, a chapter, a note for an agent).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Marker {
    /// Stable id.
    pub id: MarkerId,
    /// Position in the sequence (≥ 0).
    pub time: Flicks,
    /// Text shown next to the marker; may be empty.
    pub label: String,
    /// Colour of the marker.
    pub color: MarkerColor,
}

/// Marker colours: a named palette (the UI maps each to a design token) matching the eight marker
/// colours Premiere Pro exports, so markers survive the Premiere package (prompt 28).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum MarkerColor {
    /// Green (Premiere's default).
    #[default]
    Green,
    /// Red.
    Red,
    /// Purple.
    Purple,
    /// Orange.
    Orange,
    /// Yellow.
    Yellow,
    /// White.
    White,
    /// Blue.
    Blue,
    /// Cyan.
    Cyan,
}

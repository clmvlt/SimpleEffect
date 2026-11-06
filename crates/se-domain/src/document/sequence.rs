//! [`Sequence`]: a timeline with a format, tracks and markers.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Format, Marker, Track};
use crate::ids::SequenceId;

/// A timeline. A project may hold several (e.g. a 16:9 edit and its 9:16 version).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct Sequence {
    /// Stable id.
    pub id: SequenceId,
    /// Name shown in the UI.
    pub name: String,
    /// Frame size and rates.
    pub format: Format,
    /// Video and audio tracks; video tracks stack in this order (first = bottom).
    pub tracks: Vec<Track>,
    /// Labelled instants, in any order.
    pub markers: Vec<Marker>,
}

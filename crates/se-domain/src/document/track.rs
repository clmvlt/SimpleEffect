//! [`Track`]: a lane of a sequence holding non-overlapping items.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::Item;
use crate::ids::TrackId;

/// A horizontal lane of a sequence. Its items never overlap in time.
///
/// Video tracks are composited in their order within [`super::Sequence::tracks`]: the first video
/// track is the bottom layer (V1), each following one is drawn above it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::struct_excessive_bools,
    reason = "the four flags are independent toggles of the track header"
)]
pub struct Track {
    /// Stable id, unique in the project.
    pub id: TrackId,
    /// Video or audio; decides which items it may hold.
    pub kind: TrackKind,
    /// Name shown in the track header (`V1`, `Voix off`…).
    pub name: String,
    /// Audio of this track is silenced.
    pub muted: bool,
    /// Only solo tracks are heard when at least one track is solo.
    pub solo: bool,
    /// Items of this track cannot be edited (by the user or by agents).
    pub locked: bool,
    /// Pictures of this track are not drawn.
    pub hidden: bool,
    /// Track gain in decibels (0 = unchanged).
    pub volume_db: f64,
    /// Stereo balance from −1 (left) to 1 (right).
    pub pan: f64,
    /// The items, in any order (operations keep them sorted by start).
    pub items: Vec<Item>,
}

/// Whether a track holds pictures or sound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub enum TrackKind {
    /// Pictures: media clips, texts, shapes, images, templates, captions.
    Video,
    /// Sound: media clips of video or audio assets.
    Audio,
}

//! [`TouchedEntities`]: what an operation or a transaction added, changed or removed.

use std::collections::BTreeSet;

use crate::{AssetId, CaptionStyleId, ItemId, MarkerId, SequenceId, TrackId};

/// Ids of the entities an edit added, changed or removed.
///
/// The project actor (prompt 07) turns them into whole-entity upserts (still present) and
/// deletes (gone) for the renderer (`docs/architecture.md` §5.6); commits bump the revision of
/// touched items that still exist.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TouchedEntities {
    /// Items added, changed or removed (moving an item to another track counts).
    pub items: BTreeSet<ItemId>,
    /// Tracks added, removed or whose header changed.
    pub tracks: BTreeSet<TrackId>,
    /// Sequences added, removed, renamed, reformatted, or whose track list changed.
    pub sequences: BTreeSet<SequenceId>,
    /// Markers added, changed or removed.
    pub markers: BTreeSet<MarkerId>,
    /// Assets added, changed or removed.
    pub assets: BTreeSet<AssetId>,
    /// Caption styles added, changed or removed.
    pub caption_styles: BTreeSet<CaptionStyleId>,
    /// The project's own fields (its name) changed.
    pub project: bool,
}

impl TouchedEntities {
    /// Adds everything `other` touched.
    pub fn merge(&mut self, other: Self) {
        self.items.extend(other.items);
        self.tracks.extend(other.tracks);
        self.sequences.extend(other.sequences);
        self.markers.extend(other.markers);
        self.assets.extend(other.assets);
        self.caption_styles.extend(other.caption_styles);
        self.project |= other.project;
    }

    /// Whether nothing was touched.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
            && self.tracks.is_empty()
            && self.sequences.is_empty()
            && self.markers.is_empty()
            && self.assets.is_empty()
            && self.caption_styles.is_empty()
            && !self.project
    }
}

//! Small helpers of the generator: choosing entities of a project, building a shape item,
//! shifting times.

use se_domain::{
    Color, DesignPixels, Flicks, Item, ItemKind, Project, SequenceId, ShapeGeometry, ShapeItem,
    TimeRange, TrackId, TrackKind, Transform2D,
    test_support::{item, millis},
};

/// A new 80 × 80 white ellipse (revision 0) with test index `index`.
pub fn shape(index: u64, range: TimeRange) -> Item {
    Item {
        revision: 0,
        ..item(
            index,
            range,
            ItemKind::Shape(ShapeItem {
                geometry: ShapeGeometry::Ellipse {
                    width: DesignPixels(80.0),
                    height: DesignPixels(80.0),
                },
                fill: Color::WHITE,
                transform: Transform2D::default(),
            }),
        )
    }
}

pub fn shift(time: Flicks, offset_ms: i32) -> Flicks {
    time.checked_add(millis(i64::from(offset_ms)))
        .unwrap_or(time)
}

/// The entry of `entries` chosen by `value`, `None` when there is none.
pub fn pick<T>(entries: &[T], value: u32) -> Option<&T> {
    let len = entries.len();
    (len > 0).then(|| &entries[usize::try_from(value).unwrap_or(0) % len])
}

/// Every item with its track.
pub fn items(project: &Project) -> Vec<(TrackId, Item)> {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| track.items.iter().map(move |item| (track.id, item.clone())))
        .collect()
}

pub fn tracks(project: &Project) -> Vec<TrackId> {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .map(|track| track.id)
        .collect()
}

pub fn video_tracks(project: &Project) -> Vec<TrackId> {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .filter(|track| track.kind == TrackKind::Video)
        .map(|track| track.id)
        .collect()
}

pub fn sequences(project: &Project) -> Vec<SequenceId> {
    project
        .sequences
        .iter()
        .map(|sequence| sequence.id)
        .collect()
}

pub fn sequence_tracks(project: &Project, sequence: SequenceId) -> Vec<TrackId> {
    project
        .sequences
        .iter()
        .filter(|candidate| candidate.id == sequence)
        .flat_map(|sequence| sequence.tracks.iter().map(|track| track.id))
        .collect()
}

//! Small helpers of the generator: choosing entities of a project, building a shape item,
//! shifting times.

use se_domain::{
    Animatable, Color, DesignPixels, Flicks, Item, ItemId, ItemKind, Project, SequenceId,
    ShapeGeometry, ShapeItem, Stroke, TimeRange, TrackId, TrackKind, Transform2D, TransitionId,
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
                fill: Animatable::constant(Color::WHITE),
                stroke: Stroke::default(),
                cover_frame: false,
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

/// Every cut: two items of a track where the first ends as the second starts (track kind
/// included, to choose the transition kind).
pub fn cuts(project: &Project) -> Vec<(TrackId, TrackKind, ItemId, ItemId)> {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| {
            track.items.windows(2).filter_map(move |pair| {
                (pair[0].timeline_range.end() == pair[1].timeline_range.start())
                    .then_some((track.id, track.kind, pair[0].id, pair[1].id))
            })
        })
        .collect()
}

/// Every transition id.
pub fn transitions(project: &Project) -> Vec<TransitionId> {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| track.transitions.iter().map(|transition| transition.id))
        .collect()
}

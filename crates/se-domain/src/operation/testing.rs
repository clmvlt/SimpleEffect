//! Helpers of the operation unit tests, around `test_support::example_project()`:
//! V1 = track 1 (clips 1 `[0 s, 10 s)` and 2 `[10 s, 14 s)` at 2×), V2 = track 2 (text 3
//! `[1 s, 4 s)`, image 4 `[5 s, 7 s)`), V3 = track 3 (template 5, shape 6, caption 7), A1 = track
//! 4 (clip 8 linked to clip 1), A2 = track 5 (music 9 `[0 s, 14 s)`).

use crate::{
    Applied, DomainError, Item, ItemId, ItemKind, MediaClip, Operation, Project, SequenceId,
    ShapeGeometry, ShapeItem, TimeRange, TrackId, Transform2D, apply,
    test_support::{example_project, item},
    validate_project,
};

/// The example's only sequence.
pub(crate) const SEQUENCE: SequenceId = SequenceId::from_test_index(1);
/// Video track V1.
pub(crate) const V1: TrackId = TrackId::from_test_index(1);
/// Video track V2.
pub(crate) const V2: TrackId = TrackId::from_test_index(2);
/// Video track V3.
pub(crate) const V3: TrackId = TrackId::from_test_index(3);
/// Audio track A1.
pub(crate) const A1: TrackId = TrackId::from_test_index(4);
/// Audio track A2.
pub(crate) const A2: TrackId = TrackId::from_test_index(5);

/// The example project.
pub(crate) fn example() -> Project {
    example_project()
}

/// Applies `operation`, which must succeed, then checks that the result is valid and that
/// applying the inverse operations one by one gives back `project` exactly.
pub(crate) fn apply_ok(project: &Project, operation: impl Into<Operation>) -> Applied {
    let operation = operation.into();
    let applied =
        apply(project, &operation).unwrap_or_else(|error| panic!("{operation:?} failed: {error}"));
    assert_eq!(validate_project(&applied.project), []);
    let mut undone = applied.project.clone();
    for inverse in &applied.inverse {
        undone = apply(&undone, inverse)
            .unwrap_or_else(|error| panic!("inverse {inverse:?} failed: {error}"))
            .project;
    }
    assert_eq!(&undone, project, "the inverse did not restore the project");
    applied
}

/// Applies `operation`, which must fail, and returns the error.
pub(crate) fn apply_err(project: &Project, operation: impl Into<Operation>) -> DomainError {
    let operation = operation.into();
    match apply(project, &operation) {
        Ok(_) => panic!("{operation:?} should have failed"),
        Err(error) => error,
    }
}

/// Ids of the items of `track`, in order.
pub(crate) fn item_ids(project: &Project, track: TrackId) -> Vec<ItemId> {
    track_items(project, track)
        .iter()
        .map(|item| item.id)
        .collect()
}

/// The items of `track`.
pub(crate) fn track_items(project: &Project, track: TrackId) -> &[Item] {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .find(|candidate| candidate.id == track)
        .map(|track| track.items.as_slice())
        .expect("unknown test track")
}

/// The items of `track`, mutably.
pub(crate) fn items_of(project: &mut Project, track: TrackId) -> &mut Vec<Item> {
    project
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
        .find(|candidate| candidate.id == track)
        .map(|track| &mut track.items)
        .expect("unknown test track")
}

/// The item with test index `index`.
pub(crate) fn find(project: &Project, index: u64) -> &Item {
    let id = ItemId::from_test_index(index);
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.items)
        .find(|item| item.id == id)
        .unwrap_or_else(|| panic!("no item {index}"))
}

/// The media clip of `item`.
pub(crate) fn clip(item: &Item) -> &MediaClip {
    match &item.kind {
        ItemKind::MediaClip(clip) => clip,
        other => panic!("{} is not a media clip", other.label()),
    }
}

/// `project` with `track` locked.
pub(crate) fn lock(mut project: Project, track: TrackId) -> Project {
    project
        .sequences
        .iter_mut()
        .flat_map(|sequence| &mut sequence.tracks)
        .filter(|candidate| candidate.id == track)
        .for_each(|track| track.locked = true);
    project
}

/// A new 100 × 100 shape item (revision 0) with test index `index`.
pub(crate) fn shape_item(index: u64, range: TimeRange) -> Item {
    let shape = ShapeItem {
        geometry: ShapeGeometry::Ellipse {
            width: crate::DesignPixels(100.0),
            height: crate::DesignPixels(100.0),
        },
        fill: crate::Animatable::constant(crate::Color::WHITE),
        stroke: crate::Stroke::default(),
        cover_frame: false,
        transform: Transform2D::default(),
    };
    Item {
        revision: 0,
        ..item(index, range, ItemKind::Shape(shape))
    }
}

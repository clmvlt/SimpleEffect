//! Random operations for the property tests: a [`Seed`] of random numbers is turned into a
//! concrete operation on the current project by [`Generator::operation`], so generated
//! operations refer to ids that exist and often succeed. Failing ones are fine: a refused
//! operation must leave the project unchanged.

use std::collections::BTreeMap;

use proptest::prelude::*;
use se_domain::{
    AddAsset, AddItem, AddMarker, AddSequence, AddTrack, Animatable, AssetId, AssetPatch, Assign,
    CaptionPatch, Color, Decodability, DerivedFile, DerivedStatus, Easing, Flicks, Format,
    FormatPreset, FrameRate, InsertGap, Item, ItemEdge, ItemId, ItemKind, ItemPatch, Keyframe,
    LinkGroupId, LinkItems, Marker, MarkerColor, MarkerId, MarkerPatch, MediaClipPatch, MoveItem,
    Operation, Project, PropertyAnimation, RemoveAsset, RemoveItem, RemoveMarker, RemoveSequence,
    RemoveTrack, RenameProject, ReorderTracks, RippleDeleteRange, SceneSummary, Sequence,
    SequenceId, SequencePatch, SetKeyframes, ShapePatch, SplitItem, StreamDecodability,
    TemplateParamValue, TemplatePatch, TextPatch, TimeRange, TrackId, TrackKind, TrackPatch,
    TrimItem, UnlinkItems, UpdateAsset, UpdateItem, UpdateMarker, UpdateSequence, UpdateTrack,
    test_support::{millis, track},
};

use crate::picking::{items, pick, sequence_tracks, sequences, shape, shift, tracks, video_tracks};

/// Number of operation kinds the generator produces (every operation but `RestoreItem`, which
/// the inverses exercise).
pub const KINDS: u8 = 25;

/// Random numbers from which one operation is built.
#[derive(Debug, Clone, Copy)]
pub struct Seed {
    /// Which operation (`0..KINDS`).
    pub kind: u8,
    /// Picks an entity.
    pub pick: u32,
    /// Picks a variant or a second entity.
    pub choice: u32,
    /// A time offset in milliseconds.
    pub offset_ms: i32,
}

/// Any seed.
pub fn seed() -> impl Strategy<Value = Seed> {
    (0..KINDS, any::<u32>(), any::<u32>(), -8000..8000).prop_map(
        |(kind, pick, choice, offset_ms)| Seed {
            kind,
            pick,
            choice,
            offset_ms,
        },
    )
}

/// Builds operations, handing out fresh ids.
pub struct Generator {
    next_id: u64,
}

impl Generator {
    /// A generator whose ids do not collide with the example project's.
    pub const fn new() -> Self {
        Self { next_id: 1000 }
    }

    fn fresh(&mut self) -> u64 {
        self.next_id += 1;
        self.next_id
    }

    /// The operation of `seed` on `project`, or `None` when it has nothing to work on.
    pub fn operation(&mut self, project: &Project, seed: Seed) -> Option<Operation> {
        match seed.kind {
            0..=10 => self.item_operation(project, seed),
            11..=17 => self.structure_operation(project, seed),
            _ => self.other_operation(project, seed),
        }
    }

    fn item_operation(&mut self, project: &Project, seed: Seed) -> Option<Operation> {
        if seed.kind == 0 {
            let track = *pick(&video_tracks(project), seed.pick)?;
            let start = millis(i64::from(seed.offset_ms.unsigned_abs()));
            let duration = millis(500 + i64::from(seed.choice % 2500));
            let item = shape(self.fresh(), TimeRange::new(start, duration).ok()?);
            return Some(AddItem { track, item }.into());
        }
        if seed.kind == 9 || seed.kind == 10 {
            return self.time_operation(project, seed);
        }
        let (track, item) = pick(&items(project), seed.pick)?.clone();
        Some(match seed.kind {
            1 => RemoveItem {
                item: item.id,
                expected_revision: None,
            }
            .into(),
            2 => Self::move_item(project, &item, track, seed)?,
            3 => trim(&item, seed),
            4 => self.split(&item, seed),
            5 => update(&item, seed),
            6 => keyframes(&item, seed),
            7 => self.link(project, &item, seed),
            _ => UnlinkItems {
                items: vec![item.id],
            }
            .into(),
        })
    }

    fn move_item(project: &Project, item: &Item, track: TrackId, seed: Seed) -> Option<Operation> {
        let all = tracks(project);
        let target = if seed.choice.is_multiple_of(3) {
            *pick(&all, seed.choice)?
        } else {
            track
        };
        let start = shift(item.timeline_range.start(), seed.offset_ms).max(Flicks::ZERO);
        Some(
            MoveItem {
                item: item.id,
                track: target,
                start,
                expected_revision: Some(item.revision),
            }
            .into(),
        )
    }

    fn split(&mut self, item: &Item, seed: Seed) -> Operation {
        let range = item.timeline_range;
        let fraction = i64::from(seed.choice % 1000);
        let offset = Flicks::new(range.duration().get() / 1000 * fraction).unwrap_or(Flicks::ZERO);
        SplitItem {
            item: item.id,
            at: range.start().checked_add(offset).unwrap_or(range.start()),
            new_item_id: ItemId::from_test_index(self.fresh()),
            expected_revision: None,
        }
        .into()
    }

    fn link(&mut self, project: &Project, item: &Item, seed: Seed) -> Operation {
        let mut ids = vec![item.id];
        if let Some((_, other)) = pick(&items(project), seed.choice)
            && other.id != item.id
        {
            ids.push(other.id);
        }
        LinkItems {
            items: ids,
            group: LinkGroupId::from_test_index(self.fresh()),
        }
        .into()
    }

    fn time_operation(&mut self, project: &Project, seed: Seed) -> Option<Operation> {
        let sequence = *pick(&sequences(project), seed.pick)?;
        let start = millis(i64::from(seed.offset_ms.unsigned_abs()) * 2);
        let duration = millis(100 + i64::from(seed.choice % 3000));
        let listed = sequence_tracks(project, sequence);
        let tracks = match seed.choice % 3 {
            0 => None,
            _ => Some(vec![*pick(&listed, seed.choice / 3)?]),
        };
        let range = TimeRange::new(start, duration).ok()?;
        if seed.kind == 9 {
            let split_ids = self.split_ids(project, sequence, |r| {
                r.start() < range.start() && r.end() > range.end()
            });
            return Some(
                RippleDeleteRange {
                    sequence,
                    range,
                    tracks,
                    split_ids,
                }
                .into(),
            );
        }
        let split_ids = self.split_ids(project, sequence, |r| r.start() < start && r.end() > start);
        Some(
            InsertGap {
                sequence,
                at: start,
                duration,
                tracks,
                split_ids,
            }
            .into(),
        )
    }

    /// A fresh id for every item of `sequence` whose range matches `crosses`.
    fn split_ids(
        &mut self,
        project: &Project,
        sequence: SequenceId,
        crosses: impl Fn(TimeRange) -> bool,
    ) -> BTreeMap<ItemId, ItemId> {
        let sequence = project
            .sequences
            .iter()
            .find(|candidate| candidate.id == sequence);
        let crossing: Vec<ItemId> = sequence
            .into_iter()
            .flat_map(|sequence| &sequence.tracks)
            .flat_map(|track| &track.items)
            .filter(|item| crosses(item.timeline_range))
            .map(|item| item.id)
            .collect();
        crossing
            .into_iter()
            .map(|id| (id, ItemId::from_test_index(self.fresh())))
            .collect()
    }

    fn structure_operation(&mut self, project: &Project, seed: Seed) -> Option<Operation> {
        let sequence = *pick(&sequences(project), seed.pick)?;
        Some(match seed.kind {
            11 => {
                let kind = if seed.choice.is_multiple_of(2) {
                    TrackKind::Video
                } else {
                    TrackKind::Audio
                };
                let new = track(self.fresh(), kind, "New", Vec::new());
                let count = sequence_tracks(project, sequence).len();
                let index = usize::try_from(seed.choice).unwrap_or(0) % (count + 1);
                AddTrack {
                    sequence,
                    index: Some(index),
                    track: new,
                }
                .into()
            }
            12 => RemoveTrack {
                track: *pick(&tracks(project), seed.choice)?,
            }
            .into(),
            13 => UpdateTrack {
                track: *pick(&tracks(project), seed.choice)?,
                patch: track_patch(seed),
            }
            .into(),
            14 => {
                let mut order = sequence_tracks(project, sequence);
                let len = order.len().max(1);
                order.rotate_left(usize::try_from(seed.choice).unwrap_or(0) % len);
                ReorderTracks { sequence, order }.into()
            }
            15 => AddSequence {
                index: None,
                sequence: self.sequence(),
            }
            .into(),
            16 => RemoveSequence { sequence }.into(),
            _ => UpdateSequence {
                sequence,
                patch: sequence_patch(seed),
            }
            .into(),
        })
    }

    fn sequence(&mut self) -> Sequence {
        let id = self.fresh();
        Sequence {
            id: SequenceId::from_test_index(id),
            name: format!("Sequence {id}"),
            format: Format::from_preset(FormatPreset::Square1x1, FrameRate::FPS_30)
                .unwrap_or_else(|| unreachable!("named presets have dimensions")),
            tracks: vec![track(self.fresh(), TrackKind::Video, "V1", Vec::new())],
            markers: Vec::new(),
        }
    }

    fn other_operation(&mut self, project: &Project, seed: Seed) -> Option<Operation> {
        let markers: Vec<MarkerId> = project
            .sequences
            .iter()
            .flat_map(|sequence| sequence.markers.iter().map(|marker| marker.id))
            .collect();
        let assets: Vec<AssetId> = project.assets.iter().map(|asset| asset.id).collect();
        Some(match seed.kind {
            18 => {
                let sequence = *pick(&sequences(project), seed.pick)?;
                let marker = Marker {
                    id: MarkerId::from_test_index(self.fresh()),
                    time: millis(i64::from(seed.offset_ms.unsigned_abs())),
                    label: "Beat".to_owned(),
                    color: MarkerColor::Cyan,
                };
                AddMarker {
                    sequence,
                    index: None,
                    marker,
                }
                .into()
            }
            19 => RemoveMarker {
                marker: *pick(&markers, seed.pick)?,
            }
            .into(),
            20 => {
                let patch = MarkerPatch {
                    time: Some(millis(i64::from(seed.offset_ms))),
                    ..MarkerPatch::default()
                };
                UpdateMarker {
                    marker: *pick(&markers, seed.pick)?,
                    patch,
                }
                .into()
            }
            21 => {
                let mut asset = project.assets.first()?.clone();
                asset.id = AssetId::from_test_index(self.fresh());
                AddAsset { index: None, asset }.into()
            }
            22 => {
                let patch = AssetPatch {
                    missing: Some(seed.choice.is_multiple_of(2)),
                    fingerprint: seed
                        .choice
                        .is_multiple_of(3)
                        .then_some(Assign { value: None }),
                    decodability: seed.choice.is_multiple_of(5).then_some(Decodability {
                        video: StreamDecodability::Undecodable,
                        audio: StreamDecodability::Decodable,
                    }),
                    mezzanine: seed.choice.is_multiple_of(7).then(|| Assign {
                        value: Some(DerivedFile {
                            relative_path: "cache/media/key/mezzanine.mp4".to_owned(),
                            status: DerivedStatus::Pending,
                        }),
                    }),
                    proxy: seed
                        .choice
                        .is_multiple_of(11)
                        .then_some(Assign { value: None }),
                    scene_detection: seed.choice.is_multiple_of(13).then_some(Assign {
                        value: Some(SceneSummary { change_count: 2 }),
                    }),
                    ..AssetPatch::default()
                };
                UpdateAsset {
                    asset: *pick(&assets, seed.pick)?,
                    patch,
                }
                .into()
            }
            23 => RemoveAsset {
                asset: *pick(&assets, seed.pick)?,
            }
            .into(),
            _ => RenameProject {
                name: format!("Project {}", seed.choice % 100),
            }
            .into(),
        })
    }
}

fn trim(item: &Item, seed: Seed) -> Operation {
    let range = item.timeline_range;
    let (edge, position) = if seed.choice.is_multiple_of(2) {
        (ItemEdge::Start, range.start())
    } else {
        (ItemEdge::End, range.end())
    };
    TrimItem {
        item: item.id,
        edge,
        to: shift(position, seed.offset_ms / 4),
        expected_revision: None,
    }
    .into()
}

fn update(item: &Item, seed: Seed) -> Operation {
    let patch = match &item.kind {
        ItemKind::MediaClip(_) => ItemPatch::MediaClip(MediaClipPatch {
            fade_in: Some(millis(i64::from(seed.choice % 1500))),
            speed: (seed.choice.is_multiple_of(4)).then_some(f64::from(seed.choice % 3 + 1) * 0.5),
            ..MediaClipPatch::default()
        }),
        ItemKind::Shape(_) | ItemKind::Image(_) | ItemKind::MotionTemplate(_)
            if seed.choice.is_multiple_of(5) =>
        {
            // A patch of the wrong kind (refused), now and then.
            ItemPatch::Caption(CaptionPatch {
                text: Some("Oops".to_owned()),
            })
        }
        ItemKind::Shape(_) => ItemPatch::Shape(ShapePatch {
            fill: Some(Color::BLACK),
            ..ShapePatch::default()
        }),
        ItemKind::Caption(_) => ItemPatch::Caption(CaptionPatch {
            text: Some(format!("Caption {}", seed.choice % 10)),
        }),
        ItemKind::MotionTemplate(_) => ItemPatch::MotionTemplate(TemplatePatch {
            template_id: None,
            params: [(
                "title".to_owned(),
                seed.choice
                    .is_multiple_of(3)
                    .then(|| TemplateParamValue::Text {
                        value: "Name".to_owned(),
                    }),
            )]
            .into(),
        }),
        // Texts get a text patch; images get one too, which they refuse.
        _ => ItemPatch::Text(TextPatch {
            content: Some(format!("Text {}", seed.choice % 10)),
            ..TextPatch::default()
        }),
    };
    UpdateItem {
        item: item.id,
        patch,
        expected_revision: None,
    }
    .into()
}

fn keyframes(item: &Item, seed: Seed) -> Operation {
    let duration = item.timeline_range.duration().get();
    let count = i64::from(seed.choice % 4);
    let keyframes = (0..=count)
        .map(|index| Keyframe {
            time: Flicks::new(duration / (count + 1) * index).unwrap_or(Flicks::ZERO),
            value: f64::from(u8::try_from(index).unwrap_or(0)) / 4.0,
            easing: Easing::Linear,
        })
        .collect();
    let value = Animatable::Keyframed { keyframes };
    let animation = if seed.choice.is_multiple_of(7) {
        PropertyAnimation::VolumeDb { value }
    } else {
        PropertyAnimation::Opacity { value }
    };
    SetKeyframes {
        item: item.id,
        animation,
        expected_revision: None,
    }
    .into()
}

fn track_patch(seed: Seed) -> TrackPatch {
    TrackPatch {
        muted: Some(seed.choice.is_multiple_of(2)),
        locked: Some(seed.choice.is_multiple_of(11)),
        name: (seed.choice.is_multiple_of(3)).then(|| format!("Track {}", seed.choice % 9)),
        ..TrackPatch::default()
    }
}

fn sequence_patch(seed: Seed) -> SequencePatch {
    let preset =
        FormatPreset::NAMED[usize::try_from(seed.choice).unwrap_or(0) % FormatPreset::NAMED.len()];
    SequencePatch {
        name: Some(format!("Cut {}", seed.choice % 9)),
        format: Format::from_preset(preset, FrameRate::FPS_25),
    }
}

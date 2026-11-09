//! Random operations for the property tests: a [`Seed`] of random numbers is turned into a
//! concrete operation on the current project by [`Generator::operation`], so generated
//! operations refer to ids that exist and often succeed. Failing ones are fine: a refused
//! operation must leave the project unchanged.

use std::collections::BTreeMap;

use proptest::prelude::*;
use se_domain::{
    AddAsset, AddCaptionStyle, AddItem, AddMarker, AddSequence, AddTemplate, AddTrack,
    AddTransition, Animatable, AnimationPhase, ApplyAnimationPreset, AssetId, AssetPatch, Assign,
    CaptionAnimation, CaptionItem, CaptionPatch, CaptionStyleId, CaptionStylePatch, CaptionWord,
    Color, Decodability, DerivedFile, DerivedStatus, DesignPixels, Easing, FadeShape, Flicks,
    Format, FormatPreset, FrameRate, InsertGap, Item, ItemEdge, ItemId, ItemKind, ItemPatch,
    Keyframe, LinkGroupId, LinkItems, Marker, MarkerColor, MarkerId, MarkerPatch, MediaClipPatch,
    MotionTemplate, MoveItem, Operation, Project, PropertyAnimation, Reframe, RemoveAsset,
    RemoveCaptionStyle, RemoveItem, RemoveMarker, RemoveSequence, RemoveTemplate, RemoveTrack,
    RemoveTransition, RenameProject, ReorderTracks, ReplaceTemplate, RippleDeleteRange,
    RippleDeleteRanges, SceneSummary, Sequence, SequenceId, SequencePatch, SetBrandKit,
    SetKeyframes, SetTranscriptionSettings, ShapePatch, SplitItem, StreamDecodability, TemplateId,
    TemplateParamValue, TemplatePatch, TextPatch, TimeRange, TrackId, TrackKind, TrackPatch,
    TrackingSettings, TranscriptionSettings, Transition, TransitionAlignment, TransitionId,
    TransitionKind, TransitionPatch, TrimItem, UnlinkItems, UpdateAsset, UpdateCaptionStyle,
    UpdateItem, UpdateMarker, UpdateSequence, UpdateTrack, UpdateTransition, Vec2,
    caption_style_from_preset,
    captions::retimed_words,
    test_support::{millis, track},
};

use crate::picking::{
    cuts, items, pick, sequence_tracks, sequences, shape, shift, tracks, transitions, video_tracks,
};

/// Number of operation kinds the generator produces (every operation but `RestoreItem`, which
/// the inverses exercise).
pub const KINDS: u8 = 34;

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
            18..=24 => self.other_operation(project, seed),
            25..=27 => self.transition_operation(project, seed),
            28 => preset(project, seed),
            29 => Some(transcription_settings(seed)),
            30 => self.ranges_operation(project, seed),
            31 => self.caption_operation(project, seed),
            32 => Some(brand_kit(project, seed)),
            _ => self.template_operation(project, seed),
        }
    }

    /// Caption styles (add, change — now and then out of range —, remove — refused while used)
    /// and caption style changes.
    fn caption_operation(&mut self, project: &Project, seed: Seed) -> Option<Operation> {
        let styles: Vec<CaptionStyleId> = project
            .caption_styles
            .iter()
            .map(|style| style.id)
            .collect();
        let presets = ["tiktokBold", "minimal", "karaoke", "box"];
        Some(match seed.choice % 4 {
            0 => AddCaptionStyle {
                index: seed.pick.is_multiple_of(2).then_some(0),
                style: caption_style_from_preset(
                    presets[usize::try_from(seed.pick).unwrap_or(0) % presets.len()],
                    CaptionStyleId::from_test_index(self.fresh()),
                )
                .ok()?,
            }
            .into(),
            1 => UpdateCaptionStyle {
                style: *pick(&styles, seed.pick)?,
                patch: CaptionStylePatch {
                    font_size: Some(DesignPixels(f64::from(seed.pick % 120))),
                    max_lines: Some(seed.pick % 6),
                    animation: Some(CaptionAnimation::PopPerWord),
                    ..CaptionStylePatch::default()
                },
            }
            .into(),
            2 => RemoveCaptionStyle {
                style: *pick(&styles, seed.pick)?,
            }
            .into(),
            _ => {
                let captions: Vec<ItemId> = items(project)
                    .into_iter()
                    .filter(|(_, item)| matches!(item.kind, ItemKind::Caption(_)))
                    .map(|(_, item)| item.id)
                    .collect();
                UpdateItem {
                    item: *pick(&captions, seed.pick)?,
                    patch: ItemPatch::Caption(CaptionPatch {
                        words: None,
                        style_id: Some(*pick(&styles, seed.pick / 7)?),
                    }),
                    expected_revision: None,
                }
                .into()
            }
        })
    }

    /// Motion templates (copies added, new versions — some dropping a parameter instances use,
    /// refused at the end of the transaction —, removals refused while used) and template values
    /// (keyframed numbers, colours, now and then out of range).
    fn template_operation(&mut self, project: &Project, seed: Seed) -> Option<Operation> {
        let template = pick(&project.templates, seed.pick)?;
        Some(match seed.choice % 4 {
            0 => AddTemplate {
                index: seed.pick.is_multiple_of(2).then_some(0),
                template: MotionTemplate {
                    id: TemplateId::from_test_index(self.fresh()),
                    ..template.clone()
                },
            }
            .into(),
            1 => {
                let mut next = template.clone();
                next.version += 1;
                if seed.pick.is_multiple_of(3) {
                    next.params.retain(|param| param.name != "slide");
                }
                ReplaceTemplate { template: next }.into()
            }
            2 => RemoveTemplate {
                template: template.id,
            }
            .into(),
            _ => {
                let instances: Vec<Item> = items(project)
                    .into_iter()
                    .map(|(_, item)| item)
                    .filter(|item| matches!(item.kind, ItemKind::MotionTemplate(_)))
                    .collect();
                let item = pick(&instances, seed.pick)?;
                UpdateItem {
                    item: item.id,
                    patch: ItemPatch::MotionTemplate(TemplatePatch {
                        template_id: None,
                        params: template_values(item.timeline_range.duration(), seed),
                    }),
                    expected_revision: None,
                }
                .into()
            }
        })
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

    /// A multi-range ripple delete: two or three apart ranges, with ids for every item a range
    /// falls strictly inside.
    fn ranges_operation(&mut self, project: &Project, seed: Seed) -> Option<Operation> {
        self.multi_cut(project, seed).map(Operation::from)
    }

    /// The multi-range ripple delete of `seed` (also used by `multi_cut.rs`).
    pub fn multi_cut(&mut self, project: &Project, seed: Seed) -> Option<RippleDeleteRanges> {
        let sequence = *pick(&sequences(project), seed.pick)?;
        let count = 2 + seed.choice % 2;
        let mut start = millis(i64::from(seed.offset_ms.unsigned_abs()) / 4);
        let mut ranges = Vec::new();
        for index in 0..count {
            let duration = millis(100 + i64::from((seed.choice >> (index * 4)) % 1500));
            let range = TimeRange::new(start, duration).ok()?;
            start = range
                .end()
                .checked_add(millis(200 + i64::from(index) * 700))?;
            ranges.push(range);
        }
        let listed = sequence_tracks(project, sequence);
        let tracks = match seed.choice % 3 {
            0 => None,
            _ => Some(vec![*pick(&listed, seed.choice / 3)?]),
        };
        let items: Vec<(ItemId, TimeRange)> = project
            .sequences
            .iter()
            .filter(|candidate| candidate.id == sequence)
            .flat_map(|sequence| &sequence.tracks)
            .flat_map(|track| &track.items)
            .map(|item| (item.id, item.timeline_range))
            .collect();
        let mut split_ids = BTreeMap::new();
        for (id, range) in items {
            let inside = ranges
                .iter()
                .filter(|cut| cut.start() > range.start() && cut.end() < range.end())
                .count();
            if inside > 0 {
                let ids = (0..inside).map(|_| ItemId::from_test_index(self.fresh()));
                split_ids.insert(id, ids.collect());
            }
        }
        Some(RippleDeleteRanges {
            sequence,
            ranges,
            tracks,
            split_ids,
        })
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

impl Generator {
    /// Adds a transition on a cut (now and then of the wrong kind or too long), resizes or
    /// realigns one, or removes one.
    fn transition_operation(&mut self, project: &Project, seed: Seed) -> Option<Operation> {
        let alignment = match seed.choice % 3 {
            0 => TransitionAlignment::Center,
            1 => TransitionAlignment::Start,
            _ => TransitionAlignment::End,
        };
        let duration = millis(i64::from(50 + seed.choice % 4000));
        Some(match seed.kind {
            25 => {
                let (track, track_kind, left_item, right_item) = *pick(&cuts(project), seed.pick)?;
                let kind = if seed.choice.is_multiple_of(11) {
                    TransitionKind::for_track(match track_kind {
                        TrackKind::Video => TrackKind::Audio,
                        TrackKind::Audio => TrackKind::Video,
                    })
                } else {
                    TransitionKind::for_track(track_kind)
                };
                let transition = Transition {
                    id: TransitionId::from_test_index(self.fresh()),
                    kind,
                    left_item,
                    right_item,
                    duration,
                    alignment,
                };
                AddTransition {
                    track,
                    index: None,
                    transition,
                }
                .into()
            }
            26 => {
                let patch = TransitionPatch {
                    duration: seed.choice.is_multiple_of(2).then_some(duration),
                    alignment: seed.choice.is_multiple_of(3).then_some(alignment),
                };
                UpdateTransition {
                    transition: *pick(&transitions(project), seed.pick)?,
                    patch,
                }
                .into()
            }
            _ => RemoveTransition {
                transition: *pick(&transitions(project), seed.pick)?,
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
            fade_out_shape: seed
                .choice
                .is_multiple_of(3)
                .then_some(FadeShape::Logarithmic),
            reframe: seed.choice.is_multiple_of(2).then(|| reframe(seed)),
            ..MediaClipPatch::default()
        }),
        ItemKind::Shape(_) | ItemKind::Image(_) | ItemKind::MotionTemplate(_)
            if seed.choice.is_multiple_of(5) =>
        {
            // A patch of the wrong kind (refused), now and then.
            ItemPatch::Caption(CaptionPatch {
                words: Some(Vec::new()),
                style_id: None,
            })
        }
        ItemKind::Shape(_) => ItemPatch::Shape(ShapePatch {
            fill: Some(Color::BLACK),
            ..ShapePatch::default()
        }),
        ItemKind::Caption(caption) => ItemPatch::Caption(CaptionPatch {
            words: Some(caption_words(caption, item.timeline_range.duration(), seed)),
            style_id: None,
        }),
        ItemKind::MotionTemplate(_) => ItemPatch::MotionTemplate(TemplatePatch {
            template_id: None,
            params: [(
                "name".to_owned(),
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

/// An animation preset on a random item (refused for kinds without the preset's properties).
/// Values of the example's lower third: a keyframed slide (beyond its maximum now and then), a
/// colour, and the line reset to its default.
fn template_values(duration: Flicks, seed: Seed) -> BTreeMap<String, Option<TemplateParamValue>> {
    let end = Flicks::new(duration.get() / 2).unwrap_or(Flicks::ZERO);
    let slide = f64::from(seed.choice % 500);
    BTreeMap::from([
        (
            "slide".to_owned(),
            Some(TemplateParamValue::Number {
                value: Animatable::Keyframed {
                    keyframes: vec![
                        Keyframe {
                            time: Flicks::ZERO,
                            value: slide,
                            easing: Easing::Linear,
                        },
                        Keyframe {
                            time: end.max(Flicks::from_millis(1)),
                            value: 0.0,
                            easing: Easing::Linear,
                        },
                    ],
                },
            }),
        ),
        (
            "accent".to_owned(),
            Some(TemplateParamValue::Color {
                value: Animatable::constant(Color::opaque(0x10, 0x20, 0x30)),
            }),
        ),
        ("showLine".to_owned(), None),
    ])
}

fn preset(project: &Project, seed: Seed) -> Option<Operation> {
    const PRESETS: [(&str, AnimationPhase); 5] = [
        ("fade", AnimationPhase::In),
        ("slideFromRight", AnimationPhase::Out),
        ("pop", AnimationPhase::In),
        ("typewriter", AnimationPhase::In),
        ("bounce", AnimationPhase::Emphasis),
    ];
    let (_, item) = pick(&items(project), seed.pick)?.clone();
    let (preset, phase) = PRESETS[usize::try_from(seed.choice).unwrap_or(0) % PRESETS.len()];
    Some(
        ApplyAnimationPreset {
            item: item.id,
            preset: preset.to_owned(),
            phase,
            duration: millis(100 + i64::from(seed.choice % 2000)),
            start: seed
                .offset_ms
                .is_positive()
                .then(|| millis(i64::from(seed.offset_ms))),
            expected_revision: None,
        }
        .into(),
    )
}

/// A reframe mode; now and then a focus outside the picture or a tracking setting out of range
/// (refused by validation).
fn reframe(seed: Seed) -> Reframe {
    let x = f64::from(seed.choice % 11) / 10.0;
    let focus = Animatable::constant(Vec2::new(x, 0.5));
    match seed.choice % 5 {
        0 => Reframe::Fit,
        1 => Reframe::Custom,
        2 => Reframe::Tracked {
            focus,
            tracking: TrackingSettings {
                smoothing: f64::from(seed.choice % 13) / 10.0,
                ..TrackingSettings::default()
            },
        },
        3 if seed.choice.is_multiple_of(7) => Reframe::Fill {
            focus: Animatable::constant(Vec2::new(x + 1.5, 0.5)),
        },
        _ => Reframe::Fill { focus },
    }
}

fn keyframes(item: &Item, seed: Seed) -> Operation {
    let duration = item.timeline_range.duration().get();
    let count = i64::from(seed.choice % 4);
    let keyframes: Vec<Keyframe<f64>> = (0..=count)
        .map(|index| Keyframe {
            time: Flicks::new(duration / (count + 1) * index).unwrap_or(Flicks::ZERO),
            value: f64::from(u8::try_from(index).unwrap_or(0)) / 4.0,
            easing: Easing::Linear,
        })
        .collect();
    if seed.choice.is_multiple_of(5) {
        // Refused unless the item is a clip filling the frame.
        let focus = keyframes
            .iter()
            .map(|keyframe| Keyframe {
                time: keyframe.time,
                value: Vec2::new(keyframe.value, 1.0 - keyframe.value),
                easing: keyframe.easing,
            })
            .collect();
        return SetKeyframes {
            item: item.id,
            animation: PropertyAnimation::ReframeFocus {
                value: Animatable::Keyframed { keyframes: focus },
            },
            expected_revision: None,
        }
        .into();
    }
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

/// New words of a caption: an edited text (valid), now and then overlapping words (refused).
fn caption_words(caption: &CaptionItem, duration: Flicks, seed: Seed) -> Vec<CaptionWord> {
    if seed.choice.is_multiple_of(9) {
        return vec![
            CaptionWord::new("a", Flicks::ZERO, duration),
            CaptionWord::new("b", Flicks::ZERO, duration),
        ];
    }
    let edits = ["Bonjour à tous", "Merci", "", "Merci d'être là ce soir"];
    let text = edits[usize::try_from(seed.choice).unwrap_or(0) % edits.len()];
    let mut words = retimed_words(&caption.words, text, duration);
    if let Some(first) = words.first_mut() {
        first.emphasis = seed.pick.is_multiple_of(2);
    }
    words
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

/// New transcription settings: detection or a language (now and then a malformed one, refused),
/// with or without a vocabulary.
/// The brand kit removed, renamed, given an asset of the project as its logo (refused unless it
/// is an image) or broken (refused).
fn brand_kit(project: &Project, seed: Seed) -> Operation {
    let template = se_domain::test_support::example_project().brand_kit;
    let mut kit = project.brand_kit.clone().or(template);
    if let Some(kit) = kit.as_mut() {
        match seed.choice % 4 {
            0 => kit.name = format!("Kit {}", seed.pick % 10),
            1 => {
                if let Some(asset) = pick(&project.assets, seed.pick) {
                    kit.logos = vec![se_domain::BrandLogo {
                        name: "Logo".to_owned(),
                        asset_id: asset.id,
                    }];
                }
            }
            2 => kit.fonts.heading = String::new(),
            _ => return SetBrandKit { brand_kit: None }.into(),
        }
    }
    SetBrandKit { brand_kit: kit }.into()
}

fn transcription_settings(seed: Seed) -> Operation {
    let language = match seed.choice % 4 {
        0 => None,
        1 => Some("en".to_owned()),
        2 => Some("fr".to_owned()),
        _ => Some("Français".to_owned()),
    };
    let vocabulary = if seed.pick.is_multiple_of(2) {
        String::new()
    } else {
        format!("SimpleEffect, terme {}", seed.pick % 50)
    };
    SetTranscriptionSettings {
        settings: TranscriptionSettings {
            language,
            vocabulary,
        },
    }
    .into()
}

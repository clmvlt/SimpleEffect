//! A large, valid project built from the domain types: tracks V1–V3 and A1–A3 filled with linked
//! picture/sound clips, texts, captions and music, plus markers. Deterministic for a given seed.

use se_domain::{
    Animatable, Asset, AssetKind, CURRENT_SCHEMA_VERSION, CaptionItem, CaptionStyleId,
    DEFAULT_CAPTION_PRESET, DesignPixels, Flicks, Format, FormatPreset, FrameRate, Item, ItemId,
    ItemKind, LinkGroupId, Marker, MarkerColor, MarkerId, MediaClip, Origin, Project, ProjectId,
    ProjectSettings, Sequence, SequenceId, TaskId, TextItem, TimeRange, Track, TrackId, TrackKind,
    TranscriptionSettings, caption_style_from_preset,
    captions::{Spread, distribute_words},
    validate_project,
};
use uuid::Uuid;

use crate::DevError;

/// A small deterministic generator (64-bit LCG): the same seed gives the same project.
struct Random(u64);

impl Random {
    /// A number in `low..high` (`high > low`).
    fn between(&mut self, low: i64, high: i64) -> i64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let span = u64::try_from(high - low).unwrap_or(1).max(1);
        low + i64::try_from((self.0 >> 33) % span).unwrap_or(0)
    }
}

/// New UUIDv7 ids of each type.
struct Ids;

macro_rules! new_id {
    ($name:ident, $id:ty) => {
        fn $name(&self) -> Result<$id, DevError> {
            <$id>::from_uuid(Uuid::now_v7()).map_err(|error| DevError::Invalid(error.to_string()))
        }
    };
}

impl Ids {
    new_id!(project, ProjectId);
    new_id!(sequence, SequenceId);
    new_id!(track, TrackId);
    new_id!(item, ItemId);
    new_id!(link, LinkGroupId);
    new_id!(marker, MarkerId);
    new_id!(task, TaskId);
    new_id!(caption_style, CaptionStyleId);
}

/// What kind of item to add next, and where.
enum Slot {
    /// A picture clip on video track `video` linked to its sound on audio track `audio`.
    LinkedClip { video: usize, audio: usize },
    /// A text or caption on track V3.
    Graphic,
    /// A sound clip on track A3.
    Music,
}

/// Track indices in the sequence (document order V1, V2, V3, A1, A2, A3).
const V1: usize = 0;
const V2: usize = 1;
const V3: usize = 2;
const A1: usize = 3;
const A2: usize = 4;
const A3: usize = 5;

fn slot(index: usize) -> Slot {
    match index % 10 {
        0..=3 => Slot::LinkedClip {
            video: V1,
            audio: A1,
        },
        4 | 5 => Slot::LinkedClip {
            video: V2,
            audio: A2,
        },
        6 | 7 => Slot::Graphic,
        _ => Slot::Music,
    }
}

/// Builds the project, filling tracks until `item_count` items exist.
struct Builder {
    ids: Ids,
    random: Random,
    rate: FrameRate,
    tracks: Vec<Track>,
    /// Next free time of each track.
    cursors: [Flicks; 6],
    videos: Vec<Asset>,
    sounds: Vec<Asset>,
    task: TaskId,
    /// The style of every caption.
    caption_style: CaptionStyleId,
    count: usize,
}

impl Builder {
    fn frames(&self, count: i64) -> Flicks {
        Flicks::from_frames(count, self.rate).unwrap_or(Flicks::ZERO)
    }

    /// A clip length (1 to 6 s) and the gap before it (up to 1 s).
    fn timing(&mut self) -> (Flicks, Flicks) {
        let duration = self.random.between(25, 150);
        let gap = self.random.between(0, 25);
        (self.frames(duration), self.frames(gap))
    }

    fn origin(&self) -> Origin {
        if self.count % 9 == 4 {
            Origin::AiTask { task_id: self.task }
        } else {
            Origin::User
        }
    }

    fn push(
        &mut self,
        track: usize,
        start: Flicks,
        duration: Flicks,
        kind: ItemKind,
        link: Option<LinkGroupId>,
    ) -> Result<(), DevError> {
        let range = TimeRange::new(start, duration)
            .map_err(|error| DevError::Invalid(error.to_string()))?;
        let item = Item {
            id: self.ids.item()?,
            revision: 1,
            origin: self.origin(),
            timeline_range: range,
            link_group: link,
            kind,
        };
        self.tracks[track].items.push(item);
        self.cursors[track] = start.checked_add(duration).unwrap_or(Flicks::MAX);
        self.count += 1;
        Ok(())
    }

    /// A media clip of `asset` lasting at most `duration` (shorter if the asset is), from a
    /// random source point.
    fn clip(&mut self, asset: &Asset, duration: Flicks) -> (MediaClip, Flicks) {
        let available = asset
            .media_info
            .as_ref()
            .and_then(|info| info.duration)
            .unwrap_or(duration);
        let duration = duration.min(available);
        let spare_frames = available
            .checked_sub(duration)
            .unwrap_or(Flicks::ZERO)
            .to_frames(self.rate, se_domain::RoundingMode::Floor);
        let mut clip = MediaClip::new(asset.id);
        let start_frame = self.random.between(0, spare_frames.max(1));
        clip.source_start = self.frames(start_frame);
        (clip, duration)
    }

    fn add(&mut self, slot: &Slot) -> Result<(), DevError> {
        let (duration, gap) = self.timing();
        match *slot {
            Slot::LinkedClip { video, audio } => {
                let asset = self.videos[self.count % self.videos.len()].clone();
                let start = self.cursors[video]
                    .max(self.cursors[audio])
                    .checked_add(gap)
                    .unwrap_or(Flicks::MAX);
                let (clip, duration) = self.clip(&asset, duration);
                let link = Some(self.ids.link()?);
                self.push(
                    video,
                    start,
                    duration,
                    ItemKind::MediaClip(clip.clone()),
                    link,
                )?;
                self.push(audio, start, duration, ItemKind::MediaClip(clip), link)
            }
            Slot::Graphic => {
                let start = self.cursors[V3].checked_add(gap).unwrap_or(Flicks::MAX);
                let kind = if self.count.is_multiple_of(2) {
                    let mut text = TextItem::new(format!("Titre {}", self.count));
                    text.style.font_size = Animatable::constant(DesignPixels(72.0));
                    ItemKind::Text(text)
                } else {
                    let number = self.count.to_string();
                    ItemKind::Caption(CaptionItem {
                        words: distribute_words(
                            &["Sous-titre", number.as_str()],
                            Flicks::ZERO,
                            duration,
                            Spread::Equal,
                        ),
                        style_id: self.caption_style,
                    })
                };
                self.push(V3, start, duration, kind, None)
            }
            Slot::Music => {
                let asset = self.sounds[self.count % self.sounds.len()].clone();
                let start = self.cursors[A3].checked_add(gap).unwrap_or(Flicks::MAX);
                let (clip, duration) = self.clip(&asset, duration);
                self.push(A3, start, duration, ItemKind::MediaClip(clip), None)
            }
        }
    }

    fn markers(&mut self, count: i64) -> Result<Vec<Marker>, DevError> {
        let end = self.cursors.iter().copied().max().unwrap_or(Flicks::ZERO);
        let colors = [
            MarkerColor::Green,
            MarkerColor::Red,
            MarkerColor::Blue,
            MarkerColor::Yellow,
            MarkerColor::Purple,
            MarkerColor::Cyan,
        ];
        (0..count)
            .map(|index| {
                let time = Flicks::new(end.get() / (count + 1) * (index + 1))
                    .map_err(|error| DevError::Invalid(error.to_string()))?;
                Ok(Marker {
                    id: self.ids.marker()?,
                    time,
                    label: format!("Repère {}", index + 1),
                    color: colors[usize::try_from(index).unwrap_or(0) % colors.len()],
                })
            })
            .collect()
    }
}

fn empty_track(ids: &Ids, name: &str, kind: TrackKind) -> Result<Track, DevError> {
    Ok(Track {
        id: ids.track()?,
        kind,
        name: name.to_owned(),
        muted: false,
        solo: false,
        locked: false,
        hidden: false,
        volume_db: 0.0,
        pan: 0.0,
        items: Vec::new(),
        transitions: Vec::new(),
    })
}

/// A valid project named `name` holding `item_count` items made from `assets` (at least one video
/// with sound and one sound file), with tracks V1–V3, A1–A3 and twelve markers.
///
/// # Errors
/// Missing kinds of assets, or a document the domain refuses (a bug of this tool).
pub fn build_project(
    name: &str,
    assets: Vec<Asset>,
    item_count: usize,
    seed: u64,
) -> Result<Project, DevError> {
    let ids = Ids;
    let format = Format::from_preset(FormatPreset::Hd16x9, FrameRate::FPS_25)
        .ok_or_else(|| DevError::Invalid("the HD preset has no format".to_owned()))?;
    let videos: Vec<Asset> = assets
        .iter()
        .filter(|asset| asset.kind == AssetKind::Video)
        .cloned()
        .collect();
    let sounds: Vec<Asset> = assets
        .iter()
        .filter(|asset| asset.kind == AssetKind::Audio)
        .cloned()
        .collect();
    if videos.is_empty() || sounds.is_empty() {
        return Err(DevError::Invalid(
            "the test media must hold videos and sound files".to_owned(),
        ));
    }
    let tracks = [
        ("V1", TrackKind::Video),
        ("V2", TrackKind::Video),
        ("V3", TrackKind::Video),
        ("A1", TrackKind::Audio),
        ("A2", TrackKind::Audio),
        ("A3", TrackKind::Audio),
    ]
    .into_iter()
    .map(|(track_name, kind)| empty_track(&ids, track_name, kind))
    .collect::<Result<Vec<_>, _>>()?;
    let task = ids.task()?;
    let caption_style = ids.caption_style()?;
    let mut builder = Builder {
        ids,
        random: Random(seed),
        rate: format.frame_rate,
        tracks,
        cursors: [Flicks::ZERO; 6],
        videos,
        sounds,
        task,
        caption_style,
        count: 0,
    };
    let mut index = 0;
    while builder.count < item_count {
        let next = slot(index);
        let fits = !matches!(next, Slot::LinkedClip { .. }) || builder.count + 2 <= item_count;
        builder.add(if fits { &next } else { &Slot::Graphic })?;
        index += 1;
    }
    let markers = builder.markers(12)?;
    let project = Project {
        id: builder.ids.project()?,
        name: name.to_owned(),
        schema_version: CURRENT_SCHEMA_VERSION,
        revision: 0,
        settings: ProjectSettings {
            default_format: format,
            transcription: TranscriptionSettings::default(),
        },
        assets,
        sequences: vec![Sequence {
            id: builder.ids.sequence()?,
            name: name.to_owned(),
            format,
            tracks: builder.tracks,
            markers,
        }],
        templates: Vec::new(),
        brand_kit: None,
        caption_styles: vec![
            caption_style_from_preset(DEFAULT_CAPTION_PRESET, builder.caption_style)
                .map_err(|error| DevError::Invalid(error.to_string()))?,
        ],
    };
    let violations = validate_project(&project);
    if let Some(first) = violations.first() {
        return Err(DevError::Invalid(format!(
            "{} violations, first: {}",
            violations.len(),
            first.message
        )));
    }
    Ok(project)
}

#[cfg(test)]
mod tests {
    use se_domain::{AssetAnalysis, AssetId, Decodability, DerivedMedia, MediaInfo};

    use super::*;

    fn asset(kind: AssetKind, seconds: i64) -> Asset {
        let info = MediaInfo {
            duration: Some(Flicks::new(seconds * Flicks::PER_SECOND).unwrap()),
            container: "mp4".to_owned(),
            video_stream: None,
            audio_streams: Vec::new(),
        };
        Asset {
            id: AssetId::from_uuid(Uuid::now_v7()).unwrap(),
            original_path: format!("/media/{seconds}.mp4"),
            relative_hint: None,
            kind,
            decodability: Decodability::unchecked(&info),
            media_info: Some(info),
            fingerprint: None,
            derived: DerivedMedia::default(),
            analysis: AssetAnalysis::default(),
            missing: false,
        }
    }

    #[test]
    fn builds_a_valid_project_with_the_requested_number_of_items() {
        let assets = vec![
            asset(AssetKind::Video, 30),
            asset(AssetKind::Video, 3),
            asset(AssetKind::Audio, 60),
        ];
        for count in [1, 2, 3, 500] {
            let project = build_project("Test", assets.clone(), count, 7).unwrap();
            let items: usize = project.sequences[0]
                .tracks
                .iter()
                .map(|track| track.items.len())
                .sum();
            assert_eq!(items, count);
            assert_eq!(validate_project(&project), []);
        }
    }

    #[test]
    fn refuses_media_without_videos_or_sounds() {
        let error = build_project("Test", vec![asset(AssetKind::Audio, 10)], 10, 1).unwrap_err();
        assert!(matches!(error, DevError::Invalid(_)));
    }
}

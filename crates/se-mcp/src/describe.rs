//! Compact descriptions of tracks, items and assets shared by the read tools: what an agent needs
//! to find its way (ids, revisions, times, a label), not the whole document (`get_item` gives
//! that).

use std::path::Path;

use se_domain::{
    AnimatableProperty, Asset, AssetKind, Item, ItemKind, Origin, Project, ShapeGeometry, Track,
    TrackKind,
};
use serde::Serialize;

use crate::time_io::{RangeOutput, TimeOutput};

/// Longest text excerpt in summaries, in characters.
const EXCERPT_CHARS: usize = 80;

/// A track in listings.
#[derive(Debug, Clone, Serialize)]
pub struct TrackSummary {
    /// Its id.
    pub track_id: String,
    /// Its name (`V1`, `A2`…).
    pub name: String,
    /// `video` or `audio`.
    pub kind: &'static str,
    /// Items on it (all of them, not only those in a range).
    pub item_count: usize,
    /// Silenced.
    #[serde(skip_serializing_if = "is_false")]
    pub muted: bool,
    /// Not shown.
    #[serde(skip_serializing_if = "is_false")]
    pub hidden: bool,
    /// Not editable.
    #[serde(skip_serializing_if = "is_false")]
    pub locked: bool,
}

/// A track's summary.
#[must_use]
pub fn track_summary(track: &Track) -> TrackSummary {
    TrackSummary {
        track_id: track.id.to_string(),
        name: track.name.clone(),
        kind: track_kind_name(track.kind),
        item_count: track.items.len(),
        muted: track.muted,
        hidden: track.hidden,
        locked: track.locked,
    }
}

/// An item in listings.
#[derive(Debug, Clone, Serialize)]
pub struct ItemSummary {
    /// Its id.
    pub item_id: String,
    /// Its revision (changes with every edit of the item).
    pub revision: u64,
    /// `media_clip`, `text`, `shape`, `image`, `motion_template` or `caption`.
    pub kind: &'static str,
    /// Its track.
    pub track_id: String,
    /// First instant in the sequence.
    pub start: TimeOutput,
    /// End in the sequence (excluded).
    pub end: TimeOutput,
    /// What it shows: the media file name, the text, the shape…
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// The asset played or shown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<String>,
    /// For clips: the part of the asset played (source time).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<RangeOutput>,
    /// For clips: the playback speed, when not 1.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub speed: Option<f64>,
    /// Properties with keyframes (`opacity`, `position`, `fontSize`…).
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub animated: Vec<&'static str>,
    /// `user` or `ai_task`.
    pub origin: &'static str,
    /// The AI task that created it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<String>,
    /// Items moved and trimmed together with it share this id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_group: Option<String>,
}

/// An item's summary.
#[must_use]
pub fn item_summary(project: &Project, track: &Track, item: &Item) -> ItemSummary {
    let range = item.timeline_range;
    let (origin, task_id) = match item.origin {
        Origin::User => ("user", None),
        Origin::AiTask { task_id } => ("ai_task", Some(task_id.to_string())),
    };
    let clip = match &item.kind {
        ItemKind::MediaClip(clip) => Some(clip),
        _ => None,
    };
    ItemSummary {
        item_id: item.id.to_string(),
        revision: item.revision,
        kind: item_kind_name(&item.kind),
        track_id: track.id.to_string(),
        start: range.start().into(),
        end: range.end().into(),
        label: item_label(project, &item.kind),
        asset_id: item_asset(&item.kind).map(|id| id.to_string()),
        source: clip
            .and_then(|clip| clip.source_range(range.duration()))
            .map(Into::into),
        speed: clip
            .map(|clip| clip.speed)
            .filter(|speed| (speed - 1.0).abs() > f64::EPSILON),
        animated: animated_properties(&item.kind),
        origin,
        task_id,
        link_group: item.link_group.map(|group| group.to_string()),
    }
}

/// An asset in listings.
#[derive(Debug, Clone, Serialize)]
pub struct AssetSummary {
    /// Its id.
    pub asset_id: String,
    /// The file name of the original.
    pub name: String,
    /// `video`, `audio` or `image`.
    pub kind: &'static str,
    /// Its length (none for images).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<TimeOutput>,
    /// Picture width in pixels (upright).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Picture height in pixels (upright).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Frames per second.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fps: Option<f64>,
    /// Sound channels of the first sound stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_channels: Option<u16>,
    /// The original file cannot be found.
    #[serde(skip_serializing_if = "is_false")]
    pub missing: bool,
    /// How many items use it, in every sequence.
    pub used_by: usize,
    /// Integrated loudness (LUFS), once analysed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub integrated_lufs: Option<f64>,
    /// Silent stretches found, once analysed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub silence_count: Option<u32>,
    /// Shot changes found, once analysed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scene_changes: Option<u32>,
}

/// An asset's summary.
#[must_use]
pub fn asset_summary(project: &Project, asset: &Asset) -> AssetSummary {
    let info = asset.media_info.as_ref();
    let video = info.and_then(|info| info.video_stream.as_ref());
    let rotated = video.is_some_and(|video| video.rotation % 180 == 90);
    let (width, height) = match video {
        Some(video) if rotated => (Some(video.height), Some(video.width)),
        Some(video) => (Some(video.width), Some(video.height)),
        None => (None, None),
    };
    AssetSummary {
        asset_id: asset.id.to_string(),
        name: asset_name(asset),
        kind: asset_kind_name(asset.kind),
        duration: info.and_then(|info| info.duration).map(Into::into),
        width,
        height,
        fps: video
            .and_then(|video| video.frame_rate)
            .map(|rate| (rate.as_f64() * 1000.0).round() / 1000.0),
        audio_channels: info
            .and_then(|info| info.audio_streams.first().map(|audio| audio.channels)),
        missing: asset.missing,
        used_by: usage_count(project, asset),
        integrated_lufs: asset
            .analysis
            .audio
            .as_ref()
            .and_then(|audio| audio.integrated_lufs),
        silence_count: asset
            .analysis
            .audio
            .as_ref()
            .map(|audio| audio.silence_count),
        scene_changes: asset
            .analysis
            .scenes
            .as_ref()
            .map(|scenes| scenes.change_count),
    }
}

/// The file name of an asset's original.
#[must_use]
pub fn asset_name(asset: &Asset) -> String {
    Path::new(&asset.original_path).file_name().map_or_else(
        || asset.original_path.clone(),
        |name| name.to_string_lossy().into_owned(),
    )
}

/// The tool name of an item kind.
#[must_use]
pub const fn item_kind_name(kind: &ItemKind) -> &'static str {
    match kind {
        ItemKind::MediaClip(_) => "media_clip",
        ItemKind::Text(_) => "text",
        ItemKind::Shape(_) => "shape",
        ItemKind::Image(_) => "image",
        ItemKind::MotionTemplate(_) => "motion_template",
        ItemKind::Caption(_) => "caption",
    }
}

/// `video` or `audio`.
#[must_use]
pub const fn track_kind_name(kind: TrackKind) -> &'static str {
    match kind {
        TrackKind::Video => "video",
        TrackKind::Audio => "audio",
    }
}

/// `video`, `audio` or `image`.
#[must_use]
pub const fn asset_kind_name(kind: AssetKind) -> &'static str {
    match kind {
        AssetKind::Video => "video",
        AssetKind::Audio => "audio",
        AssetKind::Image => "image",
    }
}

/// `text` cut to [`EXCERPT_CHARS`] characters (`…` when cut), on one line.
#[must_use]
pub fn excerpt(text: &str) -> String {
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if flat.chars().count() <= EXCERPT_CHARS {
        flat
    } else {
        let cut: String = flat.chars().take(EXCERPT_CHARS - 1).collect();
        format!("{cut}…")
    }
}

/// The asset an item plays or shows.
#[must_use]
pub const fn item_asset(kind: &ItemKind) -> Option<se_domain::AssetId> {
    match kind {
        ItemKind::MediaClip(clip) => Some(clip.asset_id),
        ItemKind::Image(image) => Some(image.asset_id),
        _ => None,
    }
}

/// The properties of `kind` that have keyframes, in [`AnimatableProperty::ALL`] order.
#[must_use]
pub fn animated_properties(kind: &ItemKind) -> Vec<&'static str> {
    AnimatableProperty::ALL
        .iter()
        .filter(|property| {
            kind.property(**property)
                .is_some_and(|reference| reference.is_keyframed())
        })
        .map(|property| property.name())
        .collect()
}

fn item_label(project: &Project, kind: &ItemKind) -> Option<String> {
    let asset_label = |id| {
        project
            .assets
            .iter()
            .find(|asset| asset.id == id)
            .map(asset_name)
    };
    match kind {
        ItemKind::MediaClip(clip) => asset_label(clip.asset_id),
        ItemKind::Image(image) => asset_label(image.asset_id),
        ItemKind::Text(text) => Some(excerpt(&text.content)),
        ItemKind::Caption(caption) => Some(excerpt(&caption.text())),
        ItemKind::Shape(shape) => Some(
            match shape.geometry {
                ShapeGeometry::Rectangle { .. } => "rectangle",
                ShapeGeometry::Ellipse { .. } => "ellipse",
                ShapeGeometry::Line { .. } => "line",
            }
            .to_owned(),
        ),
        ItemKind::MotionTemplate(template) => project
            .templates
            .iter()
            .find(|candidate| candidate.id == template.template_id)
            .map(|candidate| candidate.name.clone()),
    }
}

fn usage_count(project: &Project, asset: &Asset) -> usize {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.items)
        .filter(|item| item_asset(&item.kind) == Some(asset.id))
        .count()
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "serde's skip_serializing_if passes fields by reference"
)]
const fn is_false(value: &bool) -> bool {
    !*value
}

#[cfg(test)]
mod tests {
    use se_domain::test_support::example_project;

    use super::*;

    #[test]
    fn clips_are_summarised_with_their_source_and_animation() {
        let project = example_project();
        let track = &project.sequences[0].tracks[0];
        let summary = item_summary(&project, track, &track.items[1]);
        assert_eq!(summary.kind, "media_clip");
        assert_eq!(summary.label.as_deref(), Some("interview.mp4"));
        assert_eq!(summary.speed, Some(2.0));
        assert_eq!(summary.animated, ["reframeFocus"]);
        assert_eq!(summary.origin, "ai_task");
        let source = summary.source.unwrap();
        assert_eq!(source.start.timecode, "00:00:20.000");
        assert_eq!(source.end.timecode, "00:00:28.000");
        let json = serde_json::to_value(item_summary(&project, track, &track.items[0])).unwrap();
        assert!(json.get("speed").is_none());
        assert!(json.get("task_id").is_none());
    }

    #[test]
    fn assets_are_summarised_upright_with_their_analysis() {
        let project = example_project();
        let summary = asset_summary(&project, &project.assets[0]);
        assert_eq!(summary.name, "interview.mp4");
        assert_eq!((summary.width, summary.height), (Some(1920), Some(1080)));
        assert_eq!(summary.fps, Some(29.97));
        assert_eq!(summary.integrated_lufs, Some(-16.5));
        assert_eq!(summary.scene_changes, Some(4));
        assert!(summary.used_by >= 2);
        let json = serde_json::to_value(&summary).unwrap();
        assert!(json.get("missing").is_none());
    }

    #[test]
    fn excerpts_stay_on_one_short_line() {
        assert_eq!(excerpt("Bonjour\n  tout le monde"), "Bonjour tout le monde");
        let long = "a".repeat(200);
        let cut = excerpt(&long);
        assert_eq!(cut.chars().count(), EXCERPT_CHARS);
        assert!(cut.ends_with('…'));
    }
}

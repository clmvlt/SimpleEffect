//! Stored analyses of assets as the read tools need them: loaded once per asset per call, and
//! the clips of a sequence range that play each asset.

use std::collections::HashMap;

use se_domain::{
    AnalysisKind, AnalysisResult, AssetId, AudioAnalysis, ItemKind, SceneDetection, Sequence,
    TimeRange, Track, TrackKind,
};

use crate::{context::ToolContext, output::ToolError, timing::ClipWindow};

/// A clip of the sequence that plays part of an asset inside the range asked about.
#[derive(Debug, Clone)]
pub struct RangeClip<'a> {
    /// Its track.
    pub track: &'a Track,
    /// The item.
    pub item: &'a se_domain::Item,
    /// The asset it plays.
    pub asset_id: AssetId,
    /// Where it plays its source.
    pub window: ClipWindow,
    /// Its part inside the range.
    pub visible: TimeRange,
}

/// The media clips of `kind` tracks of `sequence` that intersect `range`, in track order then
/// time.
#[must_use]
pub fn clips_in(sequence: &Sequence, kind: TrackKind, range: TimeRange) -> Vec<RangeClip<'_>> {
    let mut clips = Vec::new();
    for track in sequence.tracks.iter().filter(|track| track.kind == kind) {
        for item in &track.items {
            let ItemKind::MediaClip(clip) = &item.kind else {
                continue;
            };
            let (Some(window), Some(visible)) = (
                ClipWindow::of(item, clip),
                item.timeline_range.intersection(range),
            ) else {
                continue;
            };
            if visible.is_empty() {
                continue;
            }
            clips.push(RangeClip {
                track,
                item,
                asset_id: clip.asset_id,
                window,
                visible,
            });
        }
    }
    clips
}

/// Analyses loaded during one tool call, at most once per asset and kind.
pub struct AnalysisCache<'a> {
    context: &'a ToolContext,
    audio: HashMap<AssetId, Option<AudioAnalysis>>,
    scenes: HashMap<AssetId, Option<SceneDetection>>,
}

impl<'a> AnalysisCache<'a> {
    /// An empty cache reading through `context`.
    #[must_use]
    pub fn new(context: &'a ToolContext) -> Self {
        Self {
            context,
            audio: HashMap::new(),
            scenes: HashMap::new(),
        }
    }

    /// The audio analysis of `asset_id`; `None` while not analysed.
    ///
    /// # Errors
    /// Reading the analysis failed (project closed, storage).
    pub async fn audio(&mut self, asset_id: AssetId) -> Result<Option<&AudioAnalysis>, ToolError> {
        if !self.audio.contains_key(&asset_id) {
            let loaded = match self.load(asset_id, AnalysisKind::AudioAnalysis).await? {
                Some(AnalysisResult::AudioAnalysis(analysis)) => Some(analysis),
                _ => None,
            };
            self.audio.insert(asset_id, loaded);
        }
        Ok(self.audio.get(&asset_id).and_then(Option::as_ref))
    }

    /// The scene detection of `asset_id`; `None` while not analysed.
    ///
    /// # Errors
    /// Reading the analysis failed (project closed, storage).
    pub async fn scenes(
        &mut self,
        asset_id: AssetId,
    ) -> Result<Option<&SceneDetection>, ToolError> {
        if !self.scenes.contains_key(&asset_id) {
            let loaded = match self.load(asset_id, AnalysisKind::SceneDetection).await? {
                Some(AnalysisResult::SceneDetection(detection)) => Some(detection),
                _ => None,
            };
            self.scenes.insert(asset_id, loaded);
        }
        Ok(self.scenes.get(&asset_id).and_then(Option::as_ref))
    }

    async fn load(
        &self,
        asset_id: AssetId,
        kind: AnalysisKind,
    ) -> Result<Option<AnalysisResult>, ToolError> {
        Ok(self
            .context
            .deps
            .media
            .analysis(self.context.agent.project_id, asset_id, kind)
            .await?)
    }
}

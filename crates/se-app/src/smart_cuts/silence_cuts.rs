//! Silence cuts: the sound tracks listened to and the cuts of the silences they share.

use std::collections::BTreeSet;

use se_domain::{AssetId, ItemKind, Sequence, SilenceSettings, TimeRange, TrackId, TrackKind};
use se_protocol::SilenceCutParams;

use super::{
    CutMedia, ResolvedCuts,
    plan::{CutParams, FrameSnap, plan_cuts},
    resolve::MIN_KEEP,
    silences::sequence_silences,
};
use crate::AppError;

/// The tracks a silence cut listens to: `tracks` (sound tracks of the sequence), or every unmuted
/// and unlocked sound track holding media.
///
/// # Errors
/// A listed track that is not a sound track of the sequence.
pub fn silence_tracks(
    sequence: &Sequence,
    tracks: Option<&[TrackId]>,
) -> Result<BTreeSet<TrackId>, AppError> {
    let Some(listed) = tracks else {
        return Ok(sequence
            .tracks
            .iter()
            .filter(|track| track.kind == TrackKind::Audio && !track.muted && !track.locked)
            .filter(|track| {
                track
                    .items
                    .iter()
                    .any(|item| matches!(item.kind, ItemKind::MediaClip(_)))
            })
            .map(|track| track.id)
            .collect());
    };
    let mut chosen = BTreeSet::new();
    for id in listed {
        let track = sequence.tracks.iter().find(|track| track.id == *id);
        match track {
            Some(track) if track.kind == TrackKind::Audio => {
                chosen.insert(track.id);
            }
            Some(track) => {
                return Err(AppError::InvalidEdit {
                    reason: format!(
                        "track {} is not a sound track: silences are read on sound tracks only",
                        track.name
                    ),
                });
            }
            None => {
                return Err(se_domain::DomainError::TrackNotInSequence {
                    track: *id,
                    sequence: sequence.id,
                }
                .into());
            }
        }
    }
    Ok(chosen)
}

/// How a silence cut listens: its parameters, its tracks and the whole sequence (silences are
/// measured over it, so one crossing the range's edge keeps its full length for the minimum).
pub(super) struct SilenceListening<'a> {
    pub(super) params: SilenceCutParams,
    pub(super) tracks: Option<&'a [TrackId]>,
    pub(super) whole: TimeRange,
}

/// The silence cuts of `range`.
pub(super) fn silence_cuts(
    sequence: &Sequence,
    media: &CutMedia,
    listen: &SilenceListening<'_>,
    range: TimeRange,
) -> Result<ResolvedCuts, AppError> {
    let seeds = silence_tracks(sequence, listen.tracks)?;
    let params = listen.params;
    let settings = SilenceSettings {
        threshold_dbfs: params.threshold_dbfs,
        min_duration: params.min_silence,
    };
    let scan = sequence_silences(
        sequence,
        &seeds,
        |asset| media.levels.get(&asset).map(AsRef::as_ref),
        settings,
        listen.whole,
    );
    let cut_params = CutParams {
        padding: params.padding,
        min_cut: sequence.format.frame_rate.frame_duration(),
        min_keep: MIN_KEEP,
        frame_rate: sequence.format.frame_rate,
        snap: FrameSnap::Inward,
    };
    let unanalysed = scan_unanalysed_in(sequence, &seeds, media, range);
    Ok(ResolvedCuts {
        cuts: plan_cuts(&scan.silences, &cut_params, &scan.clips, range),
        seeds,
        word_count: 0,
        untranscribed: BTreeSet::new(),
        unanalysed,
    })
}

/// Assets of the clips of `tracks` meeting `range` without levels.
fn scan_unanalysed_in(
    sequence: &Sequence,
    tracks: &BTreeSet<TrackId>,
    media: &CutMedia,
    range: TimeRange,
) -> BTreeSet<AssetId> {
    sequence
        .tracks
        .iter()
        .filter(|track| tracks.contains(&track.id))
        .flat_map(|track| &track.items)
        .filter(|item| item.timeline_range.intersects(range))
        .filter_map(|item| match &item.kind {
            ItemKind::MediaClip(clip) if !media.levels.contains_key(&clip.asset_id) => {
                Some(clip.asset_id)
            }
            _ => None,
        })
        .collect()
}

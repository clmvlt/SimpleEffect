//! Silences of a sequence: the times where every analysed sound track is quiet, read from the
//! level envelopes of the clips' assets through each clip's window (speed included).
//!
//! A track without a clip at some instant is silent there, but time where **no** analysed track
//! has a clip is not a silence to cut (a gap, or picture only). A clip whose asset is not
//! analysed (or analysed before level envelopes existed) counts as sound: nothing is cut where
//! the sound is unknown.

use std::collections::BTreeSet;

use se_domain::{
    AssetId, ClipWindow, Flicks, ItemKind, LevelEnvelope, Sequence, SilenceSettings, TimeRange,
    TrackId,
};

/// What a scan found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SilenceScan {
    /// Silences in sequence time, sorted, at least the minimum duration long (not padded yet).
    pub silences: Vec<TimeRange>,
    /// The windows of the analysed tracks' clips inside the limit (the bounds of the cuts).
    pub clips: Vec<TimeRange>,
    /// Assets of those clips without a level envelope.
    pub unanalysed: BTreeSet<AssetId>,
}

/// The silences of `tracks` of `sequence` inside `limit`: windows quieter than
/// `settings.threshold_dbfs` on every track at once, lasting at least `settings.min_duration`.
#[must_use]
pub fn sequence_silences<'a>(
    sequence: &Sequence,
    tracks: &BTreeSet<TrackId>,
    levels_of: impl Fn(AssetId) -> Option<&'a LevelEnvelope>,
    settings: SilenceSettings,
    limit: TimeRange,
) -> SilenceScan {
    let raw = SilenceSettings {
        min_duration: Flicks::ZERO,
        ..settings
    };
    let mut scan = SilenceScan::default();
    let mut sound = Vec::new();
    for track in sequence
        .tracks
        .iter()
        .filter(|track| tracks.contains(&track.id))
    {
        for item in &track.items {
            let ItemKind::MediaClip(clip) = &item.kind else {
                continue;
            };
            let (Some(window), Some(covered)) = (
                ClipWindow::of(item, clip),
                item.timeline_range.intersection(limit),
            ) else {
                continue;
            };
            if covered.is_empty() {
                continue;
            }
            scan.clips.push(covered);
            let quiet: Vec<TimeRange> = if let Some(levels) = levels_of(clip.asset_id) {
                levels
                    .silences(raw, window.source)
                    .into_iter()
                    .filter_map(|silence| window.map_range(silence, covered))
                    .collect()
            } else {
                scan.unanalysed.insert(clip.asset_id);
                Vec::new()
            };
            sound.extend(subtract(&[covered], &quiet));
        }
    }
    scan.silences = subtract(&union(scan.clips.clone()), &union(sound))
        .into_iter()
        .filter(|silence| silence.duration() >= settings.min_duration && !silence.is_empty())
        .collect();
    scan
}

/// `ranges` sorted, overlapping or touching ones joined.
pub(super) fn union(mut ranges: Vec<TimeRange>) -> Vec<TimeRange> {
    ranges.sort_by_key(|range| range.start());
    let mut joined: Vec<TimeRange> = Vec::with_capacity(ranges.len());
    for range in ranges.into_iter().filter(|range| !range.is_empty()) {
        match joined.last_mut() {
            Some(last) if range.start() <= last.end() => {
                if range.end() > last.end()
                    && let Ok(longer) = TimeRange::from_start_end(last.start(), range.end())
                {
                    *last = longer;
                }
            }
            _ => joined.push(range),
        }
    }
    joined
}

/// The parts of `from` (sorted, apart) that `minus` does not cover.
fn subtract(from: &[TimeRange], minus: &[TimeRange]) -> Vec<TimeRange> {
    let minus = union(minus.to_vec());
    let mut left = Vec::new();
    for range in from {
        let mut cursor = range.start();
        for hole in minus.iter().filter(|hole| hole.intersects(*range)) {
            if hole.start() > cursor
                && let Ok(part) = TimeRange::from_start_end(cursor, hole.start())
            {
                left.push(part);
            }
            cursor = cursor.max(hole.end());
        }
        if cursor < range.end()
            && let Ok(part) = TimeRange::from_start_end(cursor, range.end())
        {
            left.push(part);
        }
    }
    left
}

#[cfg(test)]
mod tests {
    use se_domain::{
        Item, MediaClip, Track, TrackKind,
        test_support::{example_ids, example_project, item, track},
    };

    use super::*;

    fn ms(value: i64) -> Flicks {
        Flicks::new(value * Flicks::PER_SECOND / 1000).unwrap()
    }

    fn span(start: i64, end: i64) -> TimeRange {
        TimeRange::from_start_end(ms(start), ms(end)).unwrap()
    }

    fn sound_clip(index: u64, start: i64, end: i64, source_start: i64, speed: f64) -> Item {
        let mut clip = MediaClip::new(example_ids::VIDEO_ASSET);
        clip.source_start = ms(source_start);
        clip.speed = speed;
        item(index, span(start, end), ItemKind::MediaClip(clip))
    }

    /// Levels of 20 s at −15 dBFS with silences (source milliseconds) at −60 dBFS.
    fn levels(silences: &[(i64, i64)]) -> LevelEnvelope {
        crate::test_support::levels_with_silences(20_000, silences)
    }

    fn sequence(tracks: Vec<Track>) -> Sequence {
        let mut sequence = example_project().sequences.remove(0);
        sequence.tracks = tracks;
        sequence
    }

    fn speech() -> SilenceSettings {
        SilenceSettings {
            threshold_dbfs: -40.0,
            min_duration: ms(300),
        }
    }

    fn spans(ranges: &[TimeRange]) -> Vec<(i64, i64)> {
        let millis = |time: Flicks| time.get() * 1000 / Flicks::PER_SECOND;
        ranges
            .iter()
            .map(|range| (millis(range.start()), millis(range.end())))
            .collect()
    }

    #[test]
    fn silences_go_through_the_clip_window_and_speed() {
        // Plays source 2 s.. at 1 s for 4 s, at double speed (source 2–10 s).
        let a1 = track(
            1,
            TrackKind::Audio,
            "A1",
            vec![sound_clip(1, 1000, 5000, 2000, 2.0)],
        );
        let sequence = sequence(vec![a1]);
        let envelope = levels(&[(4000, 6000), (9000, 9400)]);
        let scan = sequence_silences(
            &sequence,
            &BTreeSet::from([TrackId::from_test_index(1)]),
            |_| Some(&envelope),
            speech(),
            span(0, 60_000),
        );
        assert_eq!(spans(&scan.silences), [(2000, 3000)], "0.2 s is too short");
        assert_eq!(spans(&scan.clips), [(1000, 5000)]);
        assert!(scan.unanalysed.is_empty());
    }

    #[test]
    fn every_analysed_track_must_be_quiet_and_unanalysed_media_counts_as_sound() {
        let a1 = track(
            1,
            TrackKind::Audio,
            "A1",
            vec![sound_clip(1, 0, 10_000, 0, 1.0)],
        );
        let a2 = track(
            2,
            TrackKind::Audio,
            "A2",
            vec![sound_clip(2, 0, 4000, 0, 1.0)],
        );
        let sequence = sequence(vec![a1, a2]);
        let first = levels(&[(1000, 3000), (6000, 8000)]);
        let second = levels(&[(2000, 4000)]);
        let both = BTreeSet::from([TrackId::from_test_index(1), TrackId::from_test_index(2)]);
        let scan = sequence_silences(&sequence, &both, |_| None, speech(), span(0, 60_000));
        assert_eq!(scan.silences, []);
        assert_eq!(scan.unanalysed.len(), 1);
        // Track 2's clip plays another asset with its own envelope.
        let mut two_assets = sequence.clone();
        if let ItemKind::MediaClip(clip) = &mut two_assets.tracks[1].items[0].kind {
            clip.asset_id = AssetId::from_test_index(77);
        }
        let scan = sequence_silences(
            &two_assets,
            &both,
            |asset| {
                if asset == AssetId::from_test_index(77) {
                    Some(&second)
                } else {
                    Some(&first)
                }
            },
            speech(),
            span(0, 60_000),
        );
        // 2–3 s quiet on both; 6–8 s quiet on A1 where A2 has no clip.
        assert_eq!(spans(&scan.silences), [(2000, 3000), (6000, 8000)]);
    }

    #[test]
    fn time_without_any_clip_is_not_a_silence_and_the_limit_bounds_everything() {
        let a1 = track(
            1,
            TrackKind::Audio,
            "A1",
            vec![
                sound_clip(1, 0, 2000, 0, 1.0),
                sound_clip(2, 5000, 9000, 5000, 1.0),
            ],
        );
        let sequence = sequence(vec![a1]);
        let envelope = levels(&[(1500, 2000), (5000, 5600), (8000, 9000)]);
        let scan = sequence_silences(
            &sequence,
            &BTreeSet::from([TrackId::from_test_index(1)]),
            |_| Some(&envelope),
            speech(),
            span(1000, 8500),
        );
        assert_eq!(
            spans(&scan.silences),
            [(1500, 2000), (5000, 5600), (8000, 8500)]
        );
        assert_eq!(spans(&scan.clips), [(1000, 2000), (5000, 8500)]);
    }
}

//! Runs the shared vectors of `docs/test-vectors/transcript-mapping.json` against
//! [`se_domain::sequence_transcript`], and checks its invariants on random cuts. The renderer's
//! mirror (`apps/desktop/src/shared/transcript/sequence-words.test.ts`) runs the same file.

// Helpers outside #[test] functions must also fail loudly on malformed vectors.
#![allow(clippy::expect_used, clippy::panic)]

use std::collections::BTreeMap;

use proptest::prelude::*;
use se_domain::{
    AssetId, Format, FormatPreset, FrameRate, ItemId, ItemKind, MediaClip, Sequence, SequenceId,
    TimeRange, TrackKind, Transcript, TranscriptSegment, TranscriptWord, sequence_transcript,
    test_support::{item, millis, track},
};
use serde::Deserialize;

const VECTORS: &str = include_str!("../../../docs/test-vectors/transcript-mapping.json");

#[derive(Deserialize)]
struct Vectors {
    transcripts: BTreeMap<String, Vec<(String, i64, i64, u32)>>,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    name: String,
    tracks: Vec<RawTrack>,
    words: Vec<(String, i64, i64, String, u32, u32)>,
    segments: Vec<(String, i64, i64, u32, u32, u32)>,
}

#[derive(Deserialize)]
struct RawTrack {
    kind: String,
    clips: Vec<RawClip>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawClip {
    item: String,
    asset: String,
    start_ms: i64,
    end_ms: i64,
    source_start_ms: i64,
    speed: f64,
}

/// Stable ids for the vectors' names (`a`, `c1`…).
#[derive(Default)]
struct Names(BTreeMap<String, u64>);

impl Names {
    fn index(&mut self, name: &str) -> u64 {
        let next = u64::try_from(self.0.len()).expect("few names") + 1;
        *self.0.entry(name.to_owned()).or_insert(next)
    }

    fn item(&mut self, name: &str) -> ItemId {
        ItemId::from_test_index(self.index(name))
    }

    fn asset(&mut self, name: &str) -> AssetId {
        AssetId::from_test_index(1000 + self.index(name))
    }
}

fn transcript(words: &[(String, i64, i64, u32)]) -> Transcript {
    let segment_count = words.iter().map(|word| word.3 + 1).max().unwrap_or(0);
    let segments = (0..segment_count)
        .map(|index| {
            let inside: Vec<_> = words.iter().filter(|word| word.3 == index).collect();
            TranscriptSegment {
                start: millis(inside.first().expect("segment has words").1),
                end: millis(inside.last().expect("segment has words").2),
            }
        })
        .collect();
    Transcript {
        language: "fr".to_owned(),
        model: "base".to_owned(),
        words: words
            .iter()
            .map(|(text, start, end, segment)| TranscriptWord {
                text: text.clone(),
                start: millis(*start),
                end: millis(*end),
                confidence: 0.9,
                segment: *segment,
            })
            .collect(),
        segments,
    }
}

fn sequence(tracks: &[RawTrack], names: &mut Names) -> Sequence {
    let tracks = tracks
        .iter()
        .enumerate()
        .map(|(index, raw)| {
            let kind = if raw.kind == "audio" {
                TrackKind::Audio
            } else {
                TrackKind::Video
            };
            let items = raw
                .clips
                .iter()
                .map(|clip| {
                    let mut media = MediaClip::new(names.asset(&clip.asset));
                    media.source_start = millis(clip.source_start_ms);
                    media.speed = clip.speed;
                    let range =
                        TimeRange::from_start_end(millis(clip.start_ms), millis(clip.end_ms))
                            .expect("valid clip range");
                    let mut placed = item(0, range, ItemKind::MediaClip(media));
                    placed.id = names.item(&clip.item);
                    placed
                })
                .collect();
            track(
                u64::try_from(index).expect("few tracks") + 1,
                kind,
                "T",
                items,
            )
        })
        .collect();
    Sequence {
        id: SequenceId::from_test_index(1),
        name: "Vectors".to_owned(),
        format: Format::from_preset(FormatPreset::Hd16x9, FrameRate::FPS_30)
            .expect("preset format"),
        tracks,
        markers: Vec::new(),
    }
}

#[test]
fn shared_vectors_map_identically() {
    let vectors: Vectors = serde_json::from_str(VECTORS).expect("readable vectors");
    let mut names = Names::default();
    let transcripts: BTreeMap<AssetId, Transcript> = vectors
        .transcripts
        .iter()
        .map(|(name, words)| (names.asset(name), transcript(words)))
        .collect();
    for case in &vectors.cases {
        let sequence = sequence(&case.tracks, &mut names);
        let mapped = sequence_transcript(&sequence, |asset| transcripts.get(&asset));
        let words: Vec<_> = mapped
            .words
            .iter()
            .map(|word| {
                (
                    word.text.clone(),
                    word.start,
                    word.end,
                    word.item_id,
                    word.word_index,
                    word.segment,
                )
            })
            .collect();
        let expected: Vec<_> = case
            .words
            .iter()
            .map(|(text, start, end, item, index, segment)| {
                (
                    text.clone(),
                    millis(*start),
                    millis(*end),
                    names.item(item),
                    *index,
                    *segment,
                )
            })
            .collect();
        assert_eq!(words, expected, "words of « {} »", case.name);
        let segments: Vec<_> = mapped
            .segments
            .iter()
            .map(|segment| {
                (
                    segment.item_id,
                    segment.start,
                    segment.end,
                    segment.first_word,
                    segment.word_count,
                    segment.source_segment,
                )
            })
            .collect();
        let expected: Vec<_> = case
            .segments
            .iter()
            .map(|(item, start, end, first, count, source)| {
                (
                    names.item(item),
                    millis(*start),
                    millis(*end),
                    *first,
                    *count,
                    *source,
                )
            })
            .collect();
        assert_eq!(segments, expected, "segments of « {} »", case.name);
    }
}

/// A transcript of `count` words of 300 ms separated by 100 ms, five words per segment.
fn regular_transcript(count: u32) -> Transcript {
    let words: Vec<(String, i64, i64, u32)> = (0..count)
        .map(|index| {
            let start = i64::from(index) * 400;
            (format!("w{index}"), start, start + 300, index / 5)
        })
        .collect();
    transcript(&words)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(300))]

    /// Cutting an asset into consecutive clips (any cut points, any order of removal) keeps words
    /// sorted, inside their clips, each source word at most once, and only words whose middle a
    /// clip plays.
    #[test]
    fn cuts_keep_words_sorted_inside_their_clips_and_unique(
        cuts in proptest::collection::btree_set(1i64..8_000, 0..6),
        removed in proptest::collection::vec(any::<bool>(), 7),
        speed_choice in 0usize..3,
    ) {
        // Speed as numerator / denominator, so clip lengths stay whole milliseconds.
        let (numerator, denominator) = [(1, 2), (1, 1), (2, 1)][speed_choice];
        let speed = f64::from(numerator) / f64::from(denominator);
        let mut names = Names::default();
        let asset = names.asset("a");
        let source = regular_transcript(20);
        let mut points = vec![0];
        points.extend(cuts.iter().copied());
        points.push(8_000);
        let mut clips = Vec::new();
        let mut cursor = 0i64;
        for (index, pair) in points.windows(2).enumerate() {
            if removed[index] {
                continue;
            }
            let length = (pair[1] - pair[0]) * i64::from(denominator) / i64::from(numerator);
            if length <= 0 {
                continue;
            }
            clips.push(RawClip {
                item: format!("c{index}"),
                asset: "a".to_owned(),
                start_ms: cursor,
                end_ms: cursor + length,
                source_start_ms: pair[0],
                speed,
            });
            cursor += length;
        }
        let sequence = sequence(&[RawTrack { kind: "audio".to_owned(), clips }], &mut names);
        let mapped = sequence_transcript(&sequence, |id| (id == asset).then_some(&source));
        let mut seen = std::collections::HashSet::new();
        for pair in mapped.words.windows(2) {
            prop_assert!(pair[0].start <= pair[1].start);
        }
        for word in &mapped.words {
            prop_assert!(seen.insert(word.word_index), "word {} twice", word.word_index);
            let clip = sequence.tracks[0].items.iter().find(|item| item.id == word.item_id).expect("its clip");
            prop_assert!(clip.timeline_range.start() <= word.start && word.end <= clip.timeline_range.end());
            prop_assert!(word.start <= word.end);
        }
        prop_assert_eq!(mapped.segments.iter().map(|segment| segment.word_count).sum::<u32>() as usize, mapped.words.len());
    }
}

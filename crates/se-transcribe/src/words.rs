//! Tokens → words: joining whisper's sub-word tokens into written words, timing them, and grouping
//! them into segments ([`build_transcript`]).
//!
//! Joining rules (tested on French in `tests/`):
//! - a token starting with a space starts a word; others continue it (`" vid" + "éos"`,
//!   `" 20" + "26"`, `" 19" + "," + "99"`, `" c" + "'" + "est" + "-" + "à" + "-" + "dire"`);
//! - a token made only of punctuation joins the previous word, keeping its leading space
//!   (French `"pensez ?"`), except opening marks (`«`, `(`, `¿`…) which start the next word;
//! - after an apostrophe the next token continues the word even after a space (`"qu'" + " il"`);
//! - special tokens (`[_BEG_]`, `[_TT_…]`) are dropped.
//!
//! Timing: with DTW times, a word starts where the alignment left the previous text token and
//! ends at its last token's DTW time; otherwise whisper's heuristic token times are used. Times
//! are mapped from processed to original audio and bounded by the speech spans when voice
//! detection ran ([`crate::voice_spans`]), then made sorted and non-overlapping. A word's
//! confidence is the mean probability of its non-punctuation tokens.

use se_domain::{Flicks, Transcript, TranscriptSegment, TranscriptWord};

use crate::{
    voice_spans::{VoiceSpan, to_original, within_speech},
    whisper_json::{RawToken, WhisperOutput},
};

/// Marks that open a phrase and attach to the next word.
const OPENING: &[char] = &['«', '(', '[', '{', '¿', '¡', '“', '‘'];

/// A word being built.
#[derive(Debug)]
struct Draft {
    bytes: Vec<u8>,
    /// Start: DTW (previous token's end) or heuristic, in processed ms.
    start: f64,
    /// End: last token's DTW or heuristic end, in processed ms.
    end: f64,
    probabilities: Vec<f64>,
    punctuation_probabilities: Vec<f64>,
    segment: usize,
}

/// Builds the transcript of `output`: words in original-audio time (ms → flicks), segments
/// recomputed from their words (empty ones dropped).
#[must_use]
pub fn build_transcript(
    output: &WhisperOutput,
    spans: &[VoiceSpan],
    language: &str,
    model: &str,
) -> Transcript {
    let drafts = join_tokens(output);
    let mut timed: Vec<(String, f64, f64, f64, usize)> = drafts
        .into_iter()
        .filter_map(|draft| {
            let text = String::from_utf8_lossy(&draft.bytes).trim().to_owned();
            if text.is_empty() {
                return None;
            }
            let start = to_original(draft.start, spans);
            let end = to_original(draft.end, spans).max(start);
            let (start, end) = within_speech(start, end, spans);
            Some((text, start, end, confidence(&draft), draft.segment))
        })
        .collect();
    make_monotonic(&mut timed);
    assemble(timed, language, model)
}

/// Joins the tokens of every segment into drafts (times still in processed ms).
fn join_tokens(output: &WhisperOutput) -> Vec<Draft> {
    let use_dtw = output
        .segments
        .iter()
        .flat_map(|segment| &segment.tokens)
        .any(|token| token.dtw_ms.is_some());
    let mut drafts: Vec<Draft> = Vec::new();
    let mut previous_dtw: Option<f64> = None;
    let mut pending_prefix: Vec<u8> = Vec::new();
    for (segment_index, segment) in output.segments.iter().enumerate() {
        let mut word_open = false;
        for token in segment.tokens.iter().filter(|token| !token.is_special()) {
            let core = trimmed(&token.bytes);
            if core.is_empty() {
                continue;
            }
            let text = String::from_utf8_lossy(core);
            let punctuation = text.chars().all(|c| !c.is_alphanumeric());
            let spaced = token.bytes.first() == Some(&b' ');
            let opening = punctuation && text.chars().all(|c| OPENING.contains(&c));
            let after_apostrophe = word_open && drafts.last().is_some_and(ends_with_apostrophe);
            // After an opening mark the next token starts a word, even without a space.
            let continues = word_open
                && pending_prefix.is_empty()
                && !opening
                && (!spaced || punctuation || after_apostrophe);
            let dtw = token.dtw_ms.map(ms);
            if continues && let Some(word) = drafts.last_mut() {
                let piece = if after_apostrophe {
                    core
                } else {
                    token.bytes.as_slice()
                };
                word.bytes.extend_from_slice(piece);
                extend_timing(word, token, dtw, use_dtw);
                record_probability(word, token, punctuation);
            } else if opening {
                pending_prefix.extend_from_slice(core);
                if text.contains('«') {
                    pending_prefix.push(b' ');
                }
            } else {
                // The segment's own offsets are already in original time (whisper maps them);
                // its first token's (often `[_BEG_]`) are in processed time like the others.
                let segment_start = segment
                    .tokens
                    .first()
                    .map_or(segment.from_ms, |first| first.from_ms);
                let start = if use_dtw {
                    previous_dtw.unwrap_or(ms(segment_start))
                } else {
                    ms(token.from_ms)
                };
                let mut bytes = std::mem::take(&mut pending_prefix);
                bytes.extend_from_slice(core);
                let mut word = Draft {
                    bytes,
                    start,
                    end: start,
                    probabilities: Vec::new(),
                    punctuation_probabilities: Vec::new(),
                    segment: segment_index,
                };
                extend_timing(&mut word, token, dtw, use_dtw);
                record_probability(&mut word, token, punctuation);
                drafts.push(word);
                word_open = true;
            }
            if dtw.is_some() {
                previous_dtw = dtw;
            }
        }
    }
    drafts
}

/// The bytes without surrounding ASCII whitespace.
fn trimmed(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map_or(start, |index| index + 1);
    &bytes[start..end]
}

fn ends_with_apostrophe(word: &Draft) -> bool {
    let text = String::from_utf8_lossy(&word.bytes);
    text.ends_with('\'') || text.ends_with('’')
}

/// The word now ends with `token`.
fn extend_timing(word: &mut Draft, token: &RawToken, dtw: Option<f64>, use_dtw: bool) {
    if use_dtw {
        if let Some(time) = dtw {
            word.end = word.end.max(time);
        }
    } else {
        word.end = word.end.max(ms(token.to_ms));
    }
}

fn record_probability(word: &mut Draft, token: &RawToken, punctuation: bool) {
    if punctuation {
        word.punctuation_probabilities.push(token.probability);
    } else {
        word.probabilities.push(token.probability);
    }
}

/// Mean probability of the word's tokens (its punctuation only when it has nothing else).
fn confidence(word: &Draft) -> f64 {
    let values = if word.probabilities.is_empty() {
        &word.punctuation_probabilities
    } else {
        &word.probabilities
    };
    if values.is_empty() {
        return 0.0;
    }
    let count = f64::from(u32::try_from(values.len()).unwrap_or(u32::MAX));
    (values.iter().sum::<f64>() / count).clamp(0.0, 1.0)
}

/// Sorted by start, never negative, each ending before the next starts.
fn make_monotonic(words: &mut [(String, f64, f64, f64, usize)]) {
    let mut floor = 0.0_f64;
    for word in words.iter_mut() {
        word.1 = word.1.max(floor);
        word.2 = word.2.max(word.1);
        floor = word.1;
    }
    for index in 1..words.len() {
        let next_start = words[index].1;
        let previous = &mut words[index - 1];
        previous.2 = previous.2.min(next_start).max(previous.1);
    }
}

/// Converts to flicks and numbers the segments that kept words.
fn assemble(words: Vec<(String, f64, f64, f64, usize)>, language: &str, model: &str) -> Transcript {
    let mut transcript = Transcript {
        language: language.to_owned(),
        model: model.to_owned(),
        words: Vec::with_capacity(words.len()),
        segments: Vec::new(),
    };
    let mut current_source: Option<usize> = None;
    for (text, start, end, confidence, source_segment) in words {
        let (start, end) = (flicks(start), flicks(end));
        if current_source != Some(source_segment) {
            transcript.segments.push(TranscriptSegment { start, end });
            current_source = Some(source_segment);
        }
        let index = transcript.segments.len() - 1;
        if let Some(segment) = transcript.segments.last_mut() {
            segment.end = segment.end.max(end);
        }
        transcript.words.push(TranscriptWord {
            text,
            start,
            end,
            confidence,
            segment: u32::try_from(index).unwrap_or(u32::MAX),
        });
    }
    transcript
}

/// Whisper's integer milliseconds as a float for timing arithmetic.
#[allow(
    clippy::cast_precision_loss,
    reason = "transcript times in milliseconds stay far below 2^52"
)]
fn ms(milliseconds: i64) -> f64 {
    milliseconds as f64
}

/// Milliseconds as flicks (rounded), zero for unrepresentable values.
fn flicks(milliseconds: f64) -> Flicks {
    Flicks::from_seconds_f64(milliseconds / 1000.0).unwrap_or(Flicks::ZERO)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::whisper_json::RawSegment;

    fn token(text: &str, from: i64, to: i64, dtw: Option<i64>) -> RawToken {
        RawToken {
            bytes: text.as_bytes().to_vec(),
            from_ms: from,
            to_ms: to,
            probability: 0.8,
            dtw_ms: dtw,
        }
    }

    fn output(tokens: Vec<RawToken>) -> WhisperOutput {
        WhisperOutput {
            language: Some("fr".to_owned()),
            segments: vec![RawSegment {
                from_ms: 0,
                to_ms: 3000,
                tokens,
            }],
        }
    }

    fn texts(transcript: &Transcript) -> Vec<&str> {
        transcript
            .words
            .iter()
            .map(|word| word.text.as_str())
            .collect()
    }

    #[test]
    fn joins_tokens_with_french_spacing_rules() {
        let tokens = [
            "[_BEG_]", " «", " Qu", "'", "est", "-", "ce", " que", " vous", " en", " pense", "z",
            " ?", " »", " qu'", " il", " a", " dit", " (", "en", " 20", "26", ")", ",", " 19", ",",
            "99", " €", ".", "[_TT_50]",
        ];
        let tokens = tokens
            .iter()
            .enumerate()
            .map(|(index, text)| {
                let at = i64::try_from(index).unwrap() * 100;
                token(text, at, at + 100, None)
            })
            .collect();
        let transcript = build_transcript(&output(tokens), &[], "fr", "base");
        assert_eq!(
            texts(&transcript),
            [
                "« Qu'est-ce",
                "que",
                "vous",
                "en",
                "pensez ? »",
                "qu'il",
                "a",
                "dit",
                "(en",
                "2026),",
                "19,99 €."
            ]
        );
        assert_eq!(transcript.problems().len(), 0);
    }

    #[test]
    fn times_words_with_dtw_and_bounds_them_by_speech() {
        let tokens = vec![
            token("[_BEG_]", 0, 0, None),
            token(" Bonjour", 20, 420, Some(360)),
            token(" à", 420, 480, Some(600)),
            token(" tous", 480, 720, Some(900)),
            token(".", 720, 920, Some(1160)),
            token(" Aujourd", 1330, 1390, Some(1500)),
            token("'", 1390, 1440, Some(1660)),
            token("hui", 1450, 1630, Some(1920)),
        ];
        let spans = crate::voice_spans::parse_voice_spans(
            "vad_segment_info: orig_start: 0.10, orig_end: 1.18, vad_start: 0.00, vad_end: 1.08\n\
             vad_segment_info: orig_start: 1.92, orig_end: 5.41, vad_start: 1.28, vad_end: 4.77",
        );
        let transcript = build_transcript(&output(tokens), &spans, "fr", "base");
        assert_eq!(texts(&transcript), ["Bonjour", "à", "tous.", "Aujourd'hui"]);
        let ms = |time: Flicks| (time.to_seconds_f64() * 1000.0).round();
        let words = &transcript.words;
        // The first word starts at its speech span (the segment starts at 0, before it).
        assert_eq!((ms(words[0].start), ms(words[0].end)), (100.0, 460.0));
        assert_eq!((ms(words[1].start), ms(words[1].end)), (460.0, 700.0));
        // « tous. » ends with its span instead of running into the pause.
        assert_eq!((ms(words[2].start), ms(words[2].end)), (700.0, 1180.0));
        // « Aujourd'hui » starts with the next span, not inside the pause.
        assert_eq!(ms(words[3].start), 1920.0);
        assert_eq!(transcript.problems().len(), 0);
    }

    #[test]
    fn uses_heuristic_times_without_dtw_and_scores_words_without_punctuation() {
        let mut tokens = vec![token(" Oui", 100, 400, None), token(".", 400, 450, None)];
        tokens[1].probability = 0.2;
        let transcript = build_transcript(&output(tokens), &[], "fr", "custom.bin");
        let word = &transcript.words[0];
        assert_eq!(word.text, "Oui.");
        assert!((word.confidence - 0.8).abs() < 1e-9);
        assert_eq!(word.end, Flicks::from_seconds_f64(0.45).unwrap());
        assert_eq!(transcript.segments.len(), 1);
    }
}

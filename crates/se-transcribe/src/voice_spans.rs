//! Speech spans found by whisper.cpp's voice detection, read from its log.
//!
//! With `--vad`, whisper.cpp transcribes only the speech, glued together ("processed" time): it
//! maps **segment** times back to the original audio, but token times and DTW times stay in
//! processed time. Each span's `vad_segment_info` log line gives both ranges, so token times are
//! mapped back piecewise linearly, and words are then bounded by the speech they belong to (the
//! DTW end of a word before a pause otherwise runs into the pause). Measured on French speech
//! with known pauses: starts within ~30 ms, ends within ~110 ms (ADR-0031).

/// One speech span: where it lies in the original audio and in the processed audio (ms).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VoiceSpan {
    /// Start in the original audio.
    pub original_start: f64,
    /// End in the original audio.
    pub original_end: f64,
    /// Start in the processed audio.
    pub processed_start: f64,
    /// End in the processed audio.
    pub processed_end: f64,
}

/// The spans of whisper.cpp's log lines
/// `whisper_vad: vad_segment_info: orig_start: 1.92, orig_end: 5.41, vad_start: 1.28, vad_end: 4.77`
/// (seconds), in order.
#[must_use]
pub fn parse_voice_spans(log: &str) -> Vec<VoiceSpan> {
    log.lines()
        .filter_map(|line| {
            let info = &line[line.find("vad_segment_info:")? + "vad_segment_info:".len()..];
            Some(VoiceSpan {
                original_start: field(info, "orig_start:")? * 1000.0,
                original_end: field(info, "orig_end:")? * 1000.0,
                processed_start: field(info, "vad_start:")? * 1000.0,
                processed_end: field(info, "vad_end:")? * 1000.0,
            })
        })
        .collect()
}

/// The number after `name` in `text`.
fn field(text: &str, name: &str) -> Option<f64> {
    let rest = text[text.find(name)? + name.len()..].trim_start();
    let end = rest
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .unwrap_or(rest.len());
    rest[..end]
        .parse()
        .ok()
        .filter(|value: &f64| value.is_finite())
}

/// `processed_ms` in the original audio: linear inside a span and between two spans, shifted
/// before the first and after the last. Unchanged without spans.
#[must_use]
pub fn to_original(processed_ms: f64, spans: &[VoiceSpan]) -> f64 {
    let points: Vec<(f64, f64)> = spans
        .iter()
        .flat_map(|span| {
            [
                (span.processed_start, span.original_start),
                (span.processed_end, span.original_end),
            ]
        })
        .collect();
    let (Some(first), Some(last)) = (points.first(), points.last()) else {
        return processed_ms;
    };
    if processed_ms <= first.0 {
        return first.1 + (processed_ms - first.0);
    }
    for pair in points.windows(2) {
        let [(from, original_from), (to, original_to)] = [pair[0], pair[1]];
        if processed_ms <= to {
            if to <= from {
                return original_to;
            }
            let ratio = (processed_ms - from) / (to - from);
            return original_from + ratio * (original_to - original_from);
        }
    }
    last.1 + (processed_ms - last.0)
}

/// `[start, end]` (original ms) bounded by the span it overlaps most (the nearest one when it
/// overlaps none). Unchanged without spans.
#[must_use]
pub fn within_speech(start: f64, end: f64, spans: &[VoiceSpan]) -> (f64, f64) {
    let best = spans.iter().max_by(|left, right| {
        overlap(start, end, left)
            .partial_cmp(&overlap(start, end, right))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let Some(span) = best else {
        return (start, end);
    };
    let bounded_start = start.clamp(span.original_start, span.original_end);
    let bounded_end = end.clamp(bounded_start, span.original_end);
    (bounded_start, bounded_end)
}

/// How much `[start, end]` overlaps `span` (negative: the distance between them).
fn overlap(start: f64, end: f64, span: &VoiceSpan) -> f64 {
    end.min(span.original_end) - start.max(span.original_start)
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOG: &str = "whisper_vad: Including segment 0: 0.10 - 1.28 (duration: 1.18)\n\
        whisper_vad: vad_segment_info: orig_start: 0.10, orig_end: 1.18, vad_start: 0.00, vad_end: 1.08\n\
        whisper_vad: vad_segment_info: orig_start: 1.92, orig_end: 5.41, vad_start: 1.28, vad_end: 4.77\n\
        whisper_vad: Created time mapping table with 20 points\n";

    fn spans() -> Vec<VoiceSpan> {
        parse_voice_spans(LOG)
    }

    fn close(left: f64, right: f64) -> bool {
        (left - right).abs() < 1e-6
    }

    #[test]
    fn reads_the_spans_of_the_log() {
        let spans = spans();
        assert_eq!(spans.len(), 2);
        assert!(close(spans[1].original_start, 1920.0) && close(spans[1].processed_end, 4770.0));
        assert_eq!(parse_voice_spans("no voice detection here").len(), 0);
    }

    #[test]
    fn maps_processed_times_back_piecewise() {
        let spans = spans();
        // Inside a span: shifted by its offset.
        assert!(close(to_original(20.0, &spans), 120.0));
        assert!(close(to_original(2000.0, &spans), 2640.0));
        // Between spans (the glued pause): from the end of one to the start of the next.
        assert!(close(to_original(1180.0, &spans), 1550.0));
        // Beyond the last: shifted like its end.
        assert!(close(to_original(5000.0, &spans), 5640.0));
        assert!(close(to_original(42.0, &[]), 42.0));
    }

    #[test]
    fn bounds_words_by_the_speech_they_belong_to() {
        let spans = spans();
        // A word whose DTW end runs into the pause after the first span.
        let (start, end) = within_speech(900.0, 1800.0, &spans);
        assert!(close(start, 900.0) && close(end, 1180.0));
        // A word starting early, in the pause before the second span.
        let (start, end) = within_speech(1700.0, 2300.0, &spans);
        assert!(close(start, 1920.0) && close(end, 2300.0));
        assert_eq!(within_speech(3.0, 4.0, &[]), (3.0, 4.0));
    }
}

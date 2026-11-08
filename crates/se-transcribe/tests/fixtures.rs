//! Words built from recorded whisper.cpp 1.9.5 output (`tests/fixtures/whisper/`): French speech
//! from Windows' Hortense voice (`bonjour-fr.flac` is its first 6 s) with apostrophes, numbers,
//! a price and punctuation, transcribed by `base` on the processor and `large-v3-turbo` on CUDA,
//! with DTW alignment and voice detection, and once without either (language detected).
//!
//! Re-record (models in `target/test-models/whisper/`): `whisper-cli -m <model> -f <16 kHz wav>
//! -l fr -oj -ojf -of <name> -pp -dtw <preset> -nfa --vad -vm ggml-silero-v6.2.0.bin 2> <name>.log`,
//! then replace local paths in both files.

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use se_domain::Transcript;
use se_transcribe::{voice_spans::parse_voice_spans, whisper_json, words::build_transcript};

fn fixture(name: &str) -> (Vec<u8>, String) {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/whisper");
    let json = std::fs::read(dir.join(format!("{name}.json"))).unwrap();
    let log = std::fs::read_to_string(dir.join(format!("{name}.log"))).unwrap();
    (json, log)
}

fn transcript(
    name: &str,
    voice_detection: bool,
) -> (Transcript, Vec<se_transcribe::voice_spans::VoiceSpan>) {
    let (json, log) = fixture(name);
    let output = whisper_json::parse(&json).unwrap();
    let spans = if voice_detection {
        parse_voice_spans(&log)
    } else {
        Vec::new()
    };
    let language = output.language.clone().unwrap();
    (build_transcript(&output, &spans, &language, "model"), spans)
}

fn texts(transcript: &Transcript) -> Vec<&str> {
    transcript
        .words
        .iter()
        .map(|word| word.text.as_str())
        .collect()
}

fn ms(time: se_domain::Flicks) -> f64 {
    time.to_seconds_f64() * 1000.0
}

#[test]
fn turbo_on_cuda_gives_french_words_with_their_punctuation() {
    let (transcript, spans) = transcript("sample-fr-turbo-vad-cuda", true);
    assert_eq!(
        texts(&transcript),
        [
            "Bonjour",
            "à",
            "tous.",
            "Aujourd'hui,",
            "on",
            "va",
            "parler",
            "de",
            "l'intelligence",
            "artificielle.",
            "En",
            "2026,",
            "j'ai",
            "monté",
            "15",
            "vidéos",
            "en",
            "3",
            "jours,",
            "c'est-à-dire",
            "5",
            "par",
            "jour.",
            "Qu'est-ce",
            "que",
            "vous",
            "en",
            "pensez ?",
            "Le",
            "prix",
            "est",
            "de",
            "19,99",
            "euros,",
            "soit",
            "20%",
            "de",
            "moins.",
            "C'est",
            "incroyable,",
            "m'a",
            "dit",
            "mon",
            "ami.",
            "Jusqu'à",
            "présent,",
            "personne",
            "n'avait",
            "fait",
            "ça."
        ]
    );
    assert_eq!(transcript.language, "fr");
    assert!(
        transcript.problems().is_empty(),
        "{:?}",
        transcript.problems()
    );
    assert_eq!(spans.len(), 7);
    // Speech starts at 0.10 s: the first word too, not at the segment's 0.
    assert!((ms(transcript.words[0].start) - 100.0).abs() < 60.0);
    for word in &transcript.words {
        let inside = spans.iter().any(|span| {
            ms(word.start) >= span.original_start - 1.0 && ms(word.end) <= span.original_end + 1.0
        });
        assert!(inside, "« {} » lies outside the speech", word.text);
    }
}

#[test]
fn base_on_the_processor_keeps_french_spacing_before_marks() {
    let (transcript, _) = transcript("sample-fr-base-vad", true);
    let words = texts(&transcript);
    for expected in [
        "tous !",
        "Aujourd'hui,",
        "trois",
        "19,99 €,",
        "20%",
        "pensez ?",
        "ça.",
    ] {
        assert!(words.contains(&expected), "{expected} in {words:?}");
    }
    assert_eq!(transcript.problems().len(), 0);
    assert!(transcript.segments.len() >= 4);
    let low = transcript
        .words
        .iter()
        .filter(|word| word.confidence < Transcript::LOW_CONFIDENCE)
        .count();
    assert!(low < transcript.words.len() / 4, "most words are sure");
}

#[test]
fn without_alignment_or_voice_detection_heuristic_times_are_used() {
    let (transcript, _) = transcript("sample-fr-base-auto", false);
    let words = texts(&transcript);
    assert_eq!(words[0], "bonjour");
    assert!(words.contains(&"c'est-à-dire") && words.contains(&"n'avait"));
    assert_eq!(transcript.problems().len(), 0);
    assert!(ms(transcript.words.last().unwrap().end) > 25_000.0);
}

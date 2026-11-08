//! A real transcription with the bundled whisper-cli (`pnpm fetch-binaries`) and the `base` model
//! of `target/test-models/whisper/` (or `SE_WHISPER_TEST_MODELS`): `bonjour-fr.flac` (6 s of
//! French speech) through `WhisperTranscriber`, with DTW alignment and voice detection when the
//! voice model is there. Skipped with a message when the program or the model is missing (CI has
//! no models).

#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use se_app::ports::{ModelChoice, ProgressSink, Transcriber, TranscriptionRequest};
use se_transcribe::{
    HttpDownloader, WhisperLocator, WhisperTranscriber,
    platform::{PLATFORM_DIR, whisper_executable},
};

#[derive(Default)]
struct Progress(Mutex<Vec<f64>>);

impl ProgressSink for Progress {
    fn report(&self, fraction: f64) {
        self.0.lock().unwrap().push(fraction);
    }
}

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

fn models_dir() -> PathBuf {
    std::env::var_os("SE_WHISPER_TEST_MODELS").map_or_else(
        || {
            repository()
                .join("target")
                .join("test-models")
                .join("whisper")
        },
        PathBuf::from,
    )
}

#[tokio::test]
async fn transcribes_french_speech_with_the_bundled_program() {
    let bundled = repository()
        .join("resources/bin")
        .join(PLATFORM_DIR)
        .join("whisper");
    let model = models_dir().join("ggml-base.bin");
    if !bundled.join(whisper_executable()).is_file() || !model.is_file() {
        eprintln!(
            "skipped: needs {} (pnpm fetch-binaries) and {} (download ggml-base.bin)",
            bundled.display(),
            model.display()
        );
        return;
    }
    let voice = models_dir().join("ggml-silero-v6.2.0.bin");
    let work = tempfile::tempdir().unwrap();
    let audio =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/whisper/bonjour-fr.flac");
    let locator = WhisperLocator::new(None, Some(bundled), false);
    let downloader = Arc::new(HttpDownloader::new().unwrap());
    let transcriber = WhisperTranscriber::new(locator, work.path().join("runtimes"), downloader, 8);
    let run_dir = work.path().join("run");
    let request = TranscriptionRequest {
        audio,
        work_dir: run_dir.clone(),
        model: ModelChoice {
            id: "base".to_owned(),
            path: model,
            alignment: Some("base".to_owned()),
        },
        voice_model: voice.is_file().then_some(voice),
        language: Some("fr".to_owned()),
        vocabulary: Some("Bonjour, intelligence artificielle".to_owned()),
        use_gpu: false,
    };
    let progress = Progress::default();
    let output = transcriber.transcribe(&request, &progress).await.unwrap();
    let text: Vec<&str> = output
        .transcript
        .words
        .iter()
        .map(|word| word.text.as_str())
        .collect();
    let joined = text.join(" ").to_lowercase();
    assert!(
        joined.contains("bonjour") && joined.contains("aujourd'hui"),
        "{joined}"
    );
    assert!(joined.contains("l'intelligence"), "{joined}");
    assert_eq!(output.transcript.problems().len(), 0);
    assert_eq!(output.transcript.model, "base");
    assert_eq!(output.backend, se_protocol::TranscriptionBackend::Cpu);
    assert!(progress.0.lock().unwrap().contains(&1.0));
    assert!(!run_dir.exists(), "the run's folder is removed");
}

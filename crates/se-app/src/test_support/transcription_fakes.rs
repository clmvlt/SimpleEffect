//! Fakes of the transcription ports: [`FakeTranscriber`] (scripted transcripts and environment),
//! [`FakeSpeechExtractor`], [`FakeModelFiles`] (an in-memory models folder) and
//! [`FakeDownloader`] (writes into it, or fails / waits on demand).

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use se_domain::{Flicks, Transcript, TranscriptSegment, TranscriptWord};
use se_protocol::TranscriptionBackend;
use tokio::sync::Notify;

use super::lock;
use crate::ports::{
    DownloadError, DownloadSink, DownloadSpec, FileDownloader, GpuRuntimeOffer, MediaError,
    ModelFile, ModelFiles, ProgressSink, SpeechAudioExtractor, Transcriber, TranscriberEnvironment,
    TranscriberError, TranscriptionOutput, TranscriptionRequest,
};

/// The path the fake environment reports for `whisper-cli`.
pub const FAKE_WHISPER_PATH: &str = "/bin/whisper/whisper-cli";

/// A small French transcript in source time: « Bonjour à tous. » (0–0.9 s) then « Ça va ? »
/// (1.5–2 s).
#[must_use]
pub fn fake_transcript(model: &str) -> Transcript {
    let millis = |ms: i64| Flicks::new(ms * 705_600).unwrap_or(Flicks::ZERO);
    let word = |text: &str, start: i64, end: i64, segment: u32| TranscriptWord {
        text: text.to_owned(),
        start: millis(start),
        end: millis(end),
        confidence: if text == "Ça" { 0.3 } else { 0.95 },
        segment,
    };
    Transcript {
        language: "fr".to_owned(),
        model: model.to_owned(),
        words: vec![
            word("Bonjour", 0, 400, 0),
            word("à", 400, 500, 0),
            word("tous.", 500, 900, 0),
            word("Ça", 1500, 1700, 1),
            word("va ?", 1700, 2000, 1),
        ],
        segments: vec![
            TranscriptSegment {
                start: millis(0),
                end: millis(900),
            },
            TranscriptSegment {
                start: millis(1500),
                end: millis(2000),
            },
        ],
    }
}

#[derive(Debug)]
struct TranscriberState {
    environment: TranscriberEnvironment,
    requests: Vec<TranscriptionRequest>,
    whisper_dir: Option<PathBuf>,
    fail_next: Option<String>,
    held: bool,
}

/// A transcriber answering [`fake_transcript`] (with the request's model) on the CPU, or the
/// environment and failures a test sets; transcriptions can be held until released.
#[derive(Debug, Clone)]
pub struct FakeTranscriber {
    state: Arc<Mutex<TranscriberState>>,
    released: Arc<Notify>,
}

impl Default for FakeTranscriber {
    fn default() -> Self {
        Self {
            state: Arc::new(Mutex::new(TranscriberState {
                environment: TranscriberEnvironment {
                    whisper: Ok(PathBuf::from(FAKE_WHISPER_PATH)),
                    gpu_name: None,
                    backend: TranscriptionBackend::Cpu,
                    gpu_runtime: None,
                },
                requests: Vec::new(),
                whisper_dir: None,
                fail_next: None,
                held: false,
            })),
            released: Arc::new(Notify::new()),
        }
    }
}

impl FakeTranscriber {
    /// Replaces what `environment()` reports.
    pub fn set_environment(&self, environment: TranscriberEnvironment) {
        lock(&self.state).environment = environment;
    }

    /// The next transcription fails with `message`.
    pub fn fail_next(&self, message: &str) {
        lock(&self.state).fail_next = Some(message.to_owned());
    }

    /// Transcriptions wait after reporting half their progress until
    /// [`FakeTranscriber::release`].
    pub fn hold(&self) {
        lock(&self.state).held = true;
    }

    /// Lets held transcriptions finish.
    pub fn release(&self) {
        lock(&self.state).held = false;
        self.released.notify_waiters();
    }

    /// Every transcription asked for.
    #[must_use]
    pub fn requests(&self) -> Vec<TranscriptionRequest> {
        lock(&self.state).requests.clone()
    }

    /// The whisper folder last set.
    #[must_use]
    pub fn whisper_dir(&self) -> Option<PathBuf> {
        lock(&self.state).whisper_dir.clone()
    }
}

#[async_trait]
impl Transcriber for FakeTranscriber {
    fn set_whisper_dir(&self, dir: Option<PathBuf>) {
        lock(&self.state).whisper_dir = dir;
    }

    async fn environment(&self) -> TranscriberEnvironment {
        lock(&self.state).environment.clone()
    }

    async fn transcribe(
        &self,
        request: &TranscriptionRequest,
        progress: &dyn ProgressSink,
    ) -> Result<TranscriptionOutput, TranscriberError> {
        let failure = {
            let mut state = lock(&self.state);
            state.requests.push(request.clone());
            state.fail_next.take()
        };
        if let Some(message) = failure {
            return Err(TranscriberError::Failed(message));
        }
        progress.report(0.5);
        loop {
            // Created before the flag is read: `notify_waiters` wakes it even if not yet polled.
            let released = self.released.notified();
            if !lock(&self.state).held {
                break;
            }
            released.await;
        }
        progress.report(1.0);
        Ok(TranscriptionOutput {
            transcript: fake_transcript(&request.model.id),
            backend: TranscriptionBackend::Cpu,
        })
    }

    async fn install_gpu_runtime(&self, progress: &dyn DownloadSink) -> Result<(), DownloadError> {
        let mut state = lock(&self.state);
        let offer = state
            .environment
            .gpu_runtime
            .as_mut()
            .ok_or(DownloadError::NotOffered)?;
        progress.received(offer.download_size, offer.download_size);
        offer.installed = true;
        state.environment.backend = TranscriptionBackend::Cuda;
        Ok(())
    }

    async fn remove_gpu_runtime(&self) -> Result<(), DownloadError> {
        let mut state = lock(&self.state);
        if let Some(offer) = state.environment.gpu_runtime.as_mut() {
            offer.installed = false;
        }
        state.environment.backend = TranscriptionBackend::Cpu;
        Ok(())
    }
}

/// A GPU pack offer for [`FakeTranscriber::set_environment`].
#[must_use]
pub fn fake_gpu_offer(installed: bool) -> GpuRuntimeOffer {
    GpuRuntimeOffer {
        name: "CUDA 12.4".to_owned(),
        download_size: 684_913_404,
        installed,
    }
}

/// An extractor recording `(source, output)` pairs.
#[derive(Debug, Clone, Default)]
pub struct FakeSpeechExtractor {
    extracted: Arc<Mutex<Vec<(PathBuf, PathBuf)>>>,
}

impl FakeSpeechExtractor {
    /// Every extraction asked for.
    #[must_use]
    pub fn extracted(&self) -> Vec<(PathBuf, PathBuf)> {
        lock(&self.extracted).clone()
    }
}

#[async_trait]
impl SpeechAudioExtractor for FakeSpeechExtractor {
    async fn extract_speech_audio(
        &self,
        source: &Path,
        output: &Path,
        progress: &dyn ProgressSink,
    ) -> Result<(), MediaError> {
        lock(&self.extracted).push((source.to_path_buf(), output.to_path_buf()));
        progress.report(1.0);
        Ok(())
    }
}

/// One file of the fake models folder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeModelFile {
    /// Its size.
    pub size: u64,
    /// Its SHA-256.
    pub sha256: String,
    /// Whether it starts like a ggml model.
    pub is_model: bool,
}

/// An in-memory folder tree of model files (and files to import).
#[derive(Debug, Clone, Default)]
pub struct FakeModelFiles {
    files: Arc<Mutex<BTreeMap<PathBuf, FakeModelFile>>>,
}

impl FakeModelFiles {
    /// Puts a file at `path`.
    pub fn put(&self, path: &Path, file: FakeModelFile) {
        lock(&self.files).insert(path.to_path_buf(), file);
    }

    /// Whether a file exists at `path`.
    #[must_use]
    pub fn exists(&self, path: &Path) -> bool {
        lock(&self.files).contains_key(path)
    }
}

#[async_trait]
impl ModelFiles for FakeModelFiles {
    async fn list(&self, dir: &Path) -> Vec<ModelFile> {
        lock(&self.files)
            .iter()
            .filter(|(path, _)| path.parent() == Some(dir))
            .filter_map(|(path, file)| {
                let name = path.file_name()?.to_string_lossy().into_owned();
                let model = path
                    .extension()
                    .is_some_and(|extension| extension.eq_ignore_ascii_case("bin"));
                model.then_some(ModelFile {
                    file_name: name,
                    size: file.size,
                })
            })
            .collect()
    }

    async fn sha256(&self, path: &Path) -> Result<String, MediaError> {
        lock(&self.files)
            .get(path)
            .map(|file| file.sha256.clone())
            .ok_or_else(|| MediaError::NotFound {
                path: path.to_path_buf(),
            })
    }

    async fn is_whisper_model(&self, path: &Path) -> bool {
        lock(&self.files)
            .get(path)
            .is_some_and(|file| file.is_model)
    }

    async fn copy(&self, source: &Path, destination: &Path) -> Result<(), MediaError> {
        let mut files = lock(&self.files);
        let file = files
            .get(source)
            .cloned()
            .ok_or_else(|| MediaError::NotFound {
                path: source.to_path_buf(),
            })?;
        files.insert(destination.to_path_buf(), file);
        Ok(())
    }

    async fn remove(&self, path: &Path) -> Result<(), MediaError> {
        lock(&self.files).remove(path);
        Ok(())
    }
}

#[derive(Debug, Default)]
struct DownloaderState {
    specs: Vec<DownloadSpec>,
    fail_next: bool,
    held: bool,
}

/// A downloader writing the expected file into a [`FakeModelFiles`] at once, or failing / waiting
/// for [`FakeDownloader::release`] on demand.
#[derive(Debug, Clone)]
pub struct FakeDownloader {
    files: FakeModelFiles,
    state: Arc<Mutex<DownloaderState>>,
    released: Arc<Notify>,
}

impl FakeDownloader {
    /// A downloader writing into `files`.
    #[must_use]
    pub fn new(files: FakeModelFiles) -> Self {
        Self {
            files,
            state: Arc::new(Mutex::new(DownloaderState::default())),
            released: Arc::new(Notify::new()),
        }
    }

    /// Every download asked for.
    #[must_use]
    pub fn specs(&self) -> Vec<DownloadSpec> {
        lock(&self.state).specs.clone()
    }

    /// The next download fails as a mismatch.
    pub fn fail_next(&self) {
        lock(&self.state).fail_next = true;
    }

    /// Downloads wait after their first progress report until [`FakeDownloader::release`].
    pub fn hold(&self) {
        lock(&self.state).held = true;
    }

    /// Lets held downloads finish.
    pub fn release(&self) {
        lock(&self.state).held = false;
        self.released.notify_waiters();
    }
}

#[async_trait]
impl FileDownloader for FakeDownloader {
    async fn download(
        &self,
        spec: &DownloadSpec,
        progress: &dyn DownloadSink,
    ) -> Result<(), DownloadError> {
        let (fail, held) = {
            let mut state = lock(&self.state);
            state.specs.push(spec.clone());
            (std::mem::take(&mut state.fail_next), state.held)
        };
        progress.received(spec.size / 2, spec.size);
        if held {
            self.released.notified().await;
        }
        if fail {
            return Err(DownloadError::Mismatch {
                url: spec.url.clone(),
            });
        }
        progress.received(spec.size, spec.size);
        progress.verifying();
        self.files.put(
            &spec.destination,
            FakeModelFile {
                size: spec.size,
                sha256: spec.sha256.clone(),
                is_model: true,
            },
        );
        Ok(())
    }
}

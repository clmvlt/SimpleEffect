//! [`WhisperTranscriber`]: the `Transcriber` port over whisper-cli.
//!
//! A run: whisper-cli on the 16 kHz WAV with full JSON output, progress lines, DTW alignment and
//! voice detection when available; its log gives progress, the backend used and the speech spans;
//! the JSON gives the tokens, joined into words ([`crate::words`]). A failed GPU run is redone on
//! the processor once. The run's folder is removed afterwards.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use se_app::ports::{
    DownloadError, DownloadSink, FileDownloader, GpuRuntimeOffer, ProgressSink, Transcriber,
    TranscriberEnvironment, TranscriberError, TranscriptionOutput, TranscriptionRequest,
};
use se_protocol::TranscriptionBackend;
use tokio::sync::OnceCell;

use crate::{
    gpu_runtime,
    locator::{Located, WhisperLocator},
    platform::{self, MISSING_LIBRARY_ADVICE, gpu_pack, gpu_query, missing_library},
    process::{first_line, run_logged, tail},
    voice_spans::parse_voice_spans,
    whisper_args::{OUTPUT_NAME, arguments},
    whisper_json, whisper_log, words,
};

/// whisper.cpp through its command-line program.
pub struct WhisperTranscriber {
    locator: WhisperLocator,
    runtimes_dir: PathBuf,
    downloader: Arc<dyn FileDownloader>,
    threads: usize,
    gpu_name: OnceCell<Option<String>>,
}

impl WhisperTranscriber {
    /// A transcriber finding whisper-cli with `locator`, keeping the GPU pack in `runtimes_dir`
    /// (downloaded with `downloader`), computing with `threads` processor threads.
    #[must_use]
    pub fn new(
        locator: WhisperLocator,
        runtimes_dir: PathBuf,
        downloader: Arc<dyn FileDownloader>,
        threads: usize,
    ) -> Self {
        Self {
            locator,
            runtimes_dir,
            downloader,
            threads: threads.clamp(1, 8),
            gpu_name: OnceCell::new(),
        }
    }

    /// The folder of the GPU pack under `runtimes_dir`, when this platform offers one.
    #[must_use]
    pub fn gpu_pack_dir(runtimes_dir: &Path) -> Option<PathBuf> {
        gpu_pack().map(|pack| gpu_runtime::pack_dir(runtimes_dir, &pack))
    }

    /// This machine's GPU, asked once.
    async fn gpu_name(&self) -> Option<String> {
        self.gpu_name
            .get_or_init(|| async {
                let (program, args) = gpu_query()?;
                let path = which::which(program).ok()?;
                first_line(&path, args).await
            })
            .await
            .clone()
    }

    /// One run of whisper-cli; the log on failure.
    async fn run(
        &self,
        located: &Located,
        request: &TranscriptionRequest,
        use_gpu: bool,
        progress: &dyn ProgressSink,
    ) -> Result<String, TranscriberError> {
        let args = arguments(request, use_gpu, self.threads);
        let mut on_line = |line: &str| {
            if let Some(fraction) = whisper_log::progress(line) {
                progress.report(fraction);
            }
        };
        let finished = run_logged(&located.path, &args, located.path.parent(), &mut on_line)
            .await
            .map_err(TranscriberError::CannotStart)?;
        if finished.success {
            return Ok(finished.log);
        }
        if missing_library(finished.code, &finished.log) {
            return Err(TranscriberError::CannotStart(
                MISSING_LIBRARY_ADVICE.to_owned(),
            ));
        }
        Err(TranscriberError::Failed(format!(
            "whisper.cpp failed (exit code {}): {}",
            finished
                .code
                .map_or_else(|| "none".to_owned(), |code| code.to_string()),
            tail(&finished.log, 4)
        )))
    }

    /// Reads the JSON output of a finished run.
    async fn read_output(
        request: &TranscriptionRequest,
        log: &str,
    ) -> Result<se_domain::Transcript, TranscriberError> {
        let file = request.work_dir.join(format!("{OUTPUT_NAME}.json"));
        let json = tokio::fs::read(&file).await.map_err(|error| {
            TranscriberError::Failed(format!(
                "whisper.cpp wrote no transcript ({}): {}",
                error,
                tail(log, 3)
            ))
        })?;
        let output = whisper_json::parse(&json).map_err(TranscriberError::Failed)?;
        let spans = if request.voice_model.is_some() {
            parse_voice_spans(log)
        } else {
            Vec::new()
        };
        if request.voice_model.is_some() && spans.is_empty() && !output.segments.is_empty() {
            tracing::warn!("voice detection ran but its spans were not found in whisper's log");
        }
        let language = output
            .language
            .clone()
            .or_else(|| request.language.clone())
            .unwrap_or_else(|| "und".to_owned());
        Ok(words::build_transcript(
            &output,
            &spans,
            &language,
            &request.model.id,
        ))
    }
}

#[async_trait]
impl Transcriber for WhisperTranscriber {
    fn set_whisper_dir(&self, dir: Option<PathBuf>) {
        self.locator.set_override_dir(dir);
    }

    async fn environment(&self) -> TranscriberEnvironment {
        let gpu_name = self.gpu_name().await;
        let installed = self.locator.gpu_pack_program().is_some();
        let gpu_runtime = gpu_pack().map(|pack| GpuRuntimeOffer {
            name: pack.name.to_owned(),
            download_size: pack.size,
            installed,
        });
        let backend = if installed && gpu_name.is_some() {
            TranscriptionBackend::Cuda
        } else {
            platform::bundled_backend()
        };
        TranscriberEnvironment {
            whisper: self.locator.locate(true).map(|located| located.path),
            gpu_name,
            backend,
            gpu_runtime,
        }
    }

    async fn transcribe(
        &self,
        request: &TranscriptionRequest,
        progress: &dyn ProgressSink,
    ) -> Result<TranscriptionOutput, TranscriberError> {
        tokio::fs::create_dir_all(&request.work_dir)
            .await
            .map_err(|error| {
                TranscriberError::Failed(format!("{}: {error}", request.work_dir.display()))
            })?;
        let located = self.locator.locate(request.use_gpu)?;
        let mut used_gpu = request.use_gpu;
        let log = match self.run(&located, request, used_gpu, progress).await {
            Err(TranscriberError::Failed(message)) if used_gpu => {
                tracing::warn!(%message, "whisper.cpp failed on the GPU; running again on the processor");
                used_gpu = false;
                self.run(&located, request, false, progress).await
            }
            other => other,
        };
        let outcome = match log {
            Ok(log) => Self::read_output(request, &log).await.map(|transcript| {
                let backend = if used_gpu {
                    whisper_log::backend_of(&log)
                } else {
                    TranscriptionBackend::Cpu
                };
                TranscriptionOutput {
                    transcript,
                    backend,
                }
            }),
            Err(error) => Err(error),
        };
        let _ = tokio::fs::remove_dir_all(&request.work_dir).await;
        outcome
    }

    async fn install_gpu_runtime(&self, progress: &dyn DownloadSink) -> Result<(), DownloadError> {
        let pack = gpu_pack().ok_or(DownloadError::NotOffered)?;
        gpu_runtime::install(
            self.downloader.as_ref(),
            &self.runtimes_dir,
            &pack,
            progress,
        )
        .await
    }

    async fn remove_gpu_runtime(&self) -> Result<(), DownloadError> {
        match gpu_pack() {
            Some(pack) => gpu_runtime::remove(&self.runtimes_dir, &pack).await,
            None => Ok(()),
        }
    }
}

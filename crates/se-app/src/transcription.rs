//! [`TranscriptionService`] (prompt 21, ADR-0031): local transcription of media with whisper.cpp.
//!
//! - **Models and GPU pack**: the catalogue ([`catalogue`]), what the models folder holds
//!   ([`models`]), downloads started only by an explicit command (the user agreed to the size in
//!   the UI), each checked by SHA-256 and reported as `transcriptionStatusChanged` events.
//! - **Preferences** in `library.db` ([`preferences`]); per-project language and vocabulary live in
//!   the document (`ProjectSettings.transcription`).
//! - **Jobs**: `JobKind::Transcription` jobs in the shared [`crate::jobs::JobQueue`] (extract the
//!   speech audio, transcribe, store the transcript in `project.db`, record the summary in the
//!   asset as a system transaction); queued on request or after an import (preference).
//! - **Reading**: an asset's transcript, its words in a sequence's time
//!   ([`se_domain::sequence_transcript`]) and full-text search, for the protocol and MCP tools.
//!
//! Like the other services it never mutates a document itself: the summary is a transaction sent
//! to the project's actor; transcripts are not document state (written to the project database
//! directly, like analyses).

mod catalogue;
mod downloads;
mod files;
mod job;
mod models;
pub mod preferences;
mod reading;
mod state;
mod transcribe;

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use async_trait::async_trait;
use se_domain::ProjectId;
use se_protocol::TranscriptionBackend;

use self::state::State;
pub use self::{
    catalogue::{CatalogueFile, CatalogueModel, Hardware, MODELS, VOICE_MODEL, recommended_model},
    reading::{MAX_SEARCH_HITS, SequenceWords},
};
use crate::{
    AppError,
    jobs::JobQueue,
    ports::{
        AppLibraryRepository, Clock, EventSink, FileDownloader, IdGenerator, ModelFiles,
        SpeechAudioExtractor, Transcriber,
    },
    project_actor::ProjectActorHandle,
    project_service::ProjectObserver,
};

/// Transaction labels (stable identifiers translated by the renderer,
/// `apps/desktop/src/app/editor/history-label.ts`).
pub mod labels {
    /// The summary of a finished transcription (system).
    pub const RECORD_TRANSCRIPT: &str = "Record transcript";
    /// The project's transcription language or vocabulary (user, undoable).
    pub const SET_TRANSCRIPTION_SETTINGS: &str = "Set transcription settings";
}

/// The ports and services transcription works with.
#[derive(Clone)]
pub struct TranscriptionServiceDeps {
    /// whisper.cpp.
    pub transcriber: Arc<dyn Transcriber>,
    /// ffmpeg's speech audio.
    pub extractor: Arc<dyn SpeechAudioExtractor>,
    /// Downloads models.
    pub downloader: Arc<dyn FileDownloader>,
    /// The models folder.
    pub model_files: Arc<dyn ModelFiles>,
    /// Runs the jobs (shared with the media service).
    pub jobs: JobQueue,
    /// Preferences (`library.db`).
    pub library: Arc<dyn AppLibraryRepository>,
    /// `transcriptionStatusChanged` events.
    pub events: Arc<dyn EventSink>,
    /// Transaction ids.
    pub ids: Arc<dyn IdGenerator>,
    /// Timestamps of transcripts.
    pub clock: Arc<dyn Clock>,
    /// `<app data>/models/whisper`.
    pub models_dir: PathBuf,
    /// Logical processors of this machine (model recommendation, threads).
    pub cpu_threads: usize,
}

/// The transcription use cases. Cheap to clone (shared state).
#[derive(Clone)]
pub struct TranscriptionService {
    inner: Arc<Inner>,
}

struct Inner {
    deps: TranscriptionServiceDeps,
    /// Open projects, as told by [`ProjectObserver`] calls.
    open: Mutex<HashMap<ProjectId, ProjectActorHandle>>,
    /// What the settings show, kept in memory so progress events are cheap to build.
    state: Mutex<State>,
    /// Where the last transcription of this session ran.
    last_backend: Arc<Mutex<Option<TranscriptionBackend>>>,
}

impl TranscriptionService {
    /// A service with no project open; call [`TranscriptionService::start`] once.
    #[must_use]
    pub fn new(deps: TranscriptionServiceDeps) -> Self {
        Self {
            inner: Arc::new(Inner {
                deps,
                open: Mutex::new(HashMap::new()),
                state: Mutex::new(State::default()),
                last_backend: Arc::new(Mutex::new(None)),
            }),
        }
    }

    /// Loads the preferences, points the transcriber at the user's whisper folder and looks at
    /// this machine (programs, GPU, models).
    ///
    /// # Errors
    /// [`AppError::Storage`] when `library.db` cannot be read.
    pub async fn start(&self) -> Result<(), AppError> {
        self.refresh().await
    }

    fn deps(&self) -> &TranscriptionServiceDeps {
        &self.inner.deps
    }

    fn state(&self) -> MutexGuard<'_, State> {
        // Plain data: a panic elsewhere cannot leave it inconsistent.
        self.inner
            .state
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    fn registry(&self) -> MutexGuard<'_, HashMap<ProjectId, ProjectActorHandle>> {
        self.inner
            .open
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
    }

    /// The actor of an open project.
    fn handle(&self, project_id: ProjectId) -> Result<ProjectActorHandle, AppError> {
        self.registry()
            .get(&project_id)
            .cloned()
            .ok_or(AppError::ProjectNotOpen { project_id })
    }
}

#[async_trait]
impl ProjectObserver for TranscriptionService {
    async fn project_opened(&self, handle: &ProjectActorHandle) {
        self.registry().insert(handle.project_id(), handle.clone());
    }

    async fn project_closing(&self, project_id: ProjectId) {
        // Its jobs are cancelled by the media service (one queue for both).
        self.registry().remove(&project_id);
    }
}

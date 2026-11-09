//! [`AppError`]: why a use case failed, and its protocol code.

use std::path::PathBuf;

use se_domain::{
    AssetId, AssetKind, BrandKitId, DomainError, DomainErrorCode, Flicks, JobId, LibraryTemplateId,
    ProjectId, TaskId,
};
use se_protocol::{ErrorCode, ProtocolError, TemplateCheckStage, TemplateProblem};

use crate::ports::{MediaError, StorageError};

/// A failed use case. Messages are actionable: they reach users and AI agents unchanged.
#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum AppError {
    /// A persistence port failed.
    #[error(transparent)]
    Storage(#[from] StorageError),
    /// The project is not open in this core.
    #[error("project {project_id} is not open: open it first (openProject)")]
    ProjectNotOpen {
        /// The project.
        project_id: ProjectId,
    },
    /// Another folder holding a project with the same id is open (a copy made outside the app).
    #[error(
        "{} holds project {project_id}, which is already open from {}: it is probably a copy \
         made outside SimpleEffect; close the other one first, or use « Duplicate » to make copies",
        path.display(),
        open_path.display()
    )]
    SameProjectOpen {
        /// The project id both folders hold.
        project_id: ProjectId,
        /// The folder being opened.
        path: PathBuf,
        /// The folder already open.
        open_path: PathBuf,
    },
    /// A name was empty.
    #[error("the {what} name must not be empty: type a name")]
    EmptyName {
        /// What was being named (`project`, `sequence`).
        what: &'static str,
    },
    /// A sequence format breaks a document rule.
    #[error("invalid format: {details}")]
    InvalidFormat {
        /// The violated rules, as validation messages.
        details: String,
    },
    /// The journal cannot be replayed on top of the snapshot.
    #[error(
        "the journal of {} cannot be replayed: {reason}; project.json and its backups \
         (project.json.bak.1–3) are intact",
        path.display()
    )]
    JournalBroken {
        /// The project folder.
        path: PathBuf,
        /// Which entry failed and why.
        reason: String,
    },
    /// The loaded document breaks document invariants (e.g. edited by hand).
    #[error(
        "{} holds an invalid document: {details}; restore a backup (project.json.bak.1–3) or fix \
         the file",
        path.display()
    )]
    InvalidDocument {
        /// The project folder.
        path: PathBuf,
        /// The violated rules.
        details: String,
    },
    /// The document refused the change.
    #[error(transparent)]
    Domain(#[from] DomainError),
    /// The project's actor stopped while the command was waiting for it.
    #[error("project {project_id} was closed while the command was running: open it again")]
    ActorStopped {
        /// The project.
        project_id: ProjectId,
    },
    /// A media port failed (file missing, unreadable, ffmpeg absent…).
    #[error(transparent)]
    Media(#[from] MediaError),
    /// A subtitle file could not be read, written or understood.
    #[error(transparent)]
    Subtitles(#[from] crate::ports::SubtitleError),
    /// The open project has no such asset.
    #[error("project {project_id} has no asset {asset_id}: refresh the media bin")]
    AssetNotFound {
        /// The project.
        project_id: ProjectId,
        /// The asset.
        asset_id: AssetId,
    },
    /// A replacement file holds another kind of media than the asset it should replace.
    #[error(
        "{} is {found:?} media but the asset is {expected:?} media: pick a file of the same kind",
        path.display()
    )]
    MediaKindMismatch {
        /// The replacement file.
        path: PathBuf,
        /// Kind of the asset.
        expected: AssetKind,
        /// Kind of the file.
        found: AssetKind,
    },
    /// No active job has this id.
    #[error("no queued or running job {job_id}: it may have finished already")]
    JobNotFound {
        /// The job.
        job_id: JobId,
    },
    /// A mezzanine or proxy was requested for an asset it does not apply to.
    #[error("asset {asset_id} gets no {derived}: {reason}")]
    DerivedNotApplicable {
        /// The asset.
        asset_id: AssetId,
        /// `mezzanine` or `proxy`.
        derived: &'static str,
        /// Why, and what to do instead.
        reason: String,
    },
    /// A time outside an asset's media.
    #[error(
        "{:.3} s is outside asset {asset_id}, which lasts {:.3} s: pick a time between 0 and its end",
        time.to_seconds_f64(),
        duration.to_seconds_f64()
    )]
    MediaTimeOutOfRange {
        /// The asset.
        asset_id: AssetId,
        /// The time asked for.
        time: Flicks,
        /// Its media's duration.
        duration: Flicks,
    },
    /// A setting value is not accepted.
    #[error("invalid setting {name}: {reason}")]
    InvalidSetting {
        /// The setting.
        name: &'static str,
        /// What is accepted.
        reason: String,
    },
    /// An item edit that cannot be done as asked (nothing listed, several sequences, a gap that
    /// is not there any more…).
    #[error("{reason}")]
    InvalidEdit {
        /// What is wrong and what to do instead.
        reason: String,
    },
    /// A ripple edit would cut content of a track it must shift.
    #[error(
        "this ripple edit would also cut what track {track_name} holds between {:.3} s and \
         {:.3} s (tracks holding linked items shift together so they stay in sync): move or trim \
         that content first, or lock track {track_name} to leave it out of ripple edits",
        start.to_seconds_f64(),
        end.to_seconds_f64()
    )]
    RippleBlocked {
        /// Name of the track.
        track_name: String,
        /// Start of the content that would be cut.
        start: Flicks,
        /// Its end.
        end: Flicks,
    },
    /// No listed cut has media to cross-fade.
    #[error(
        "there is no media to cross-fade at this cut: both items need media beyond their edges (trim them to free some), must last long enough, and the cut must not already have a transition"
    )]
    TransitionNoRoom,
    /// The project has no such AI task.
    #[error("project {project_id} has no AI task {task_id}: refresh the task list")]
    TaskNotFound {
        /// The project.
        project_id: ProjectId,
        /// The task.
        task_id: TaskId,
    },
    /// Cancelling a task that already finished.
    #[error("AI task {task_id} is not queued or running: it has already finished")]
    TaskNotActive {
        /// The task.
        task_id: TaskId,
    },
    /// Retrying or following up a task that has not finished.
    #[error(
        "AI task {task_id} is still queued or running: wait for it to finish, or cancel it first"
    )]
    TaskStillActive {
        /// The task.
        task_id: TaskId,
    },
    /// A follow-up of a task whose agent session cannot be resumed.
    #[error("AI task {task_id} cannot be continued: {reason}")]
    TaskNotResumable {
        /// The task.
        task_id: TaskId,
        /// Why, and what to do instead.
        reason: String,
    },
    /// A task request that cannot be accepted (empty prompt, scope outside the project…).
    #[error("invalid AI task: {reason}")]
    InvalidTask {
        /// What is wrong and what to do instead.
        reason: String,
    },
    /// The AI provider cannot run tasks (CLI missing or logged out).
    #[error("AI tasks cannot run: {reason}")]
    AiUnavailable {
        /// Why, and what to do.
        reason: String,
    },
    /// No editor window can draw frames for an agent (none open, or none answered in time).
    #[error("frames cannot be drawn: {reason}")]
    RendererUnavailable {
        /// Why, and what to do.
        reason: String,
    },
    /// The renderer could not draw the requested frames.
    #[error("the editor could not draw the frames: {reason}")]
    RenderFailed {
        /// The renderer's explanation.
        reason: String,
    },
    /// A frame request or upload that cannot be accepted (too many frames, not a JPEG…).
    #[error("invalid frame render: {reason}")]
    InvalidRender {
        /// What is wrong.
        reason: String,
    },
    /// A motion template module cannot be used (prompt 24): every problem, with its line.
    #[error("{}", describe_template_problems(.problems))]
    TemplateInvalid {
        /// What the renderer or the core found.
        problems: Vec<TemplateProblem>,
    },
    /// Nothing waits for that template check any more.
    #[error("no template check {request_id} is waiting: it already finished or timed out")]
    TemplateCheckNotFound {
        /// The request.
        request_id: String,
    },
    /// Nothing waits for frames of that request any more.
    #[error("no frame render {request_id} is waiting: it already finished, failed or timed out")]
    RenderRequestNotFound {
        /// The request.
        request_id: String,
    },
    /// An AI agent's edit reaches outside its task's scope (sequence, time range or tracks).
    #[error("{reason}")]
    OutOfScope {
        /// What lies outside the scope, the scope itself and what to do instead.
        reason: String,
    },
    /// Reverting an AI task that has no open changeset.
    #[error(
        "AI task {task_id} has nothing to revert: it changed nothing since it started or was \
         last reverted"
    )]
    NothingToRevert {
        /// The task.
        task_id: TaskId,
    },
    /// Transcription cannot run (whisper.cpp missing or unable to start).
    #[error("transcription is unavailable: {reason}")]
    TranscriptionUnavailable {
        /// Why, and what to do.
        reason: String,
    },
    /// Faces cannot be detected on this machine (automatic reframing, prompt 26).
    #[error("automatic reframing is unavailable: {reason}")]
    VisionUnavailable {
        /// Why, and what to do.
        reason: String,
    },
    /// No usable transcription model is installed.
    #[error(
        "no transcription model is installed: download one in « Réglages → Transcription » (or \
         import a ggml file)"
    )]
    NoTranscriptionModel,
    /// No model, download or GPU pack matches the request.
    #[error("{what} does not exist; see « Réglages → Transcription » for what is available")]
    TranscriptionTargetNotFound {
        /// What was asked.
        what: String,
    },
    /// A file that is not a whisper.cpp model.
    #[error(
        "{path} is not a whisper.cpp model (ggml .bin file); choose a file such as ggml-base.bin"
    )]
    InvalidModelFile {
        /// The file.
        path: String,
    },
    /// A download that failed or did not match.
    #[error("{reason}")]
    DownloadFailed {
        /// What failed and what to do.
        reason: String,
    },
    /// An asset without sound asked for a transcription.
    #[error("asset {asset_id} has no sound to transcribe")]
    NoSpeechStream {
        /// The asset.
        asset_id: AssetId,
    },
    /// No entry of the template library has this id (prompt 25).
    #[error(
        "the template library has no entry {template_id}: it was removed meanwhile; list the \
         library again"
    )]
    LibraryTemplateNotFound {
        /// The entry.
        template_id: LibraryTemplateId,
    },
    /// No brand kit of the library has this id (prompt 25).
    #[error("there is no brand kit {kit_id}: it was removed meanwhile; list the kits again")]
    BrandKitNotFound {
        /// The kit.
        kit_id: BrandKitId,
    },
    /// A file the library cannot use: not a `.setemplate` archive, not an image (prompt 25).
    #[error("{path} cannot be used: {reason}")]
    InvalidLibraryFile {
        /// The file.
        path: String,
        /// Why, and what to do.
        reason: String,
    },
    /// Library information or a library kit with unusable values (prompt 25).
    #[error("{reason}")]
    InvalidLibraryEntry {
        /// Each faulty field and its rule.
        reason: String,
    },
    /// A bug: an invariant of the application itself was broken.
    #[error("internal error: {0}; please report it")]
    Internal(String),
}

impl AppError {
    /// The stable protocol code of this error.
    #[must_use]
    pub fn code(&self) -> ErrorCode {
        match self {
            Self::Storage(error) => storage_code(error),
            Self::ProjectNotOpen { .. } | Self::ActorStopped { .. } => ErrorCode::ProjectNotOpen,
            Self::SameProjectOpen { .. } => ErrorCode::ProjectLocked,
            Self::EmptyName { .. }
            | Self::InvalidFormat { .. }
            | Self::InvalidSetting { .. }
            | Self::InvalidTask { .. }
            | Self::InvalidRender { .. } => ErrorCode::InvalidRequest,
            Self::DerivedNotApplicable { .. } => ErrorCode::DerivedNotApplicable,
            Self::MediaTimeOutOfRange { .. } => ErrorCode::MediaTimeOutOfRange,
            Self::JournalBroken { .. } | Self::InvalidDocument { .. } => {
                ErrorCode::ProjectCorrupted
            }
            Self::Domain(error) => match error.code() {
                DomainErrorCode::NothingToUndo => ErrorCode::NothingToUndo,
                DomainErrorCode::NothingToRedo => ErrorCode::NothingToRedo,
                _ => ErrorCode::EditRejected,
            },
            Self::Media(error) => media_code(error),
            Self::AssetNotFound { .. } => ErrorCode::AssetNotFound,
            Self::MediaKindMismatch { .. } => ErrorCode::MediaKindMismatch,
            Self::JobNotFound { .. } => ErrorCode::JobNotFound,
            Self::InvalidEdit { .. } => ErrorCode::EditRejected,
            Self::RippleBlocked { .. } => ErrorCode::RippleBlocked,
            Self::TransitionNoRoom => ErrorCode::TransitionNoRoom,
            Self::TaskNotFound { .. } => ErrorCode::TaskNotFound,
            Self::TaskNotActive { .. } => ErrorCode::TaskNotActive,
            Self::TaskStillActive { .. } => ErrorCode::TaskStillActive,
            Self::TaskNotResumable { .. } => ErrorCode::TaskNotResumable,
            Self::AiUnavailable { .. } => ErrorCode::AiUnavailable,
            Self::RendererUnavailable { .. } => ErrorCode::RendererUnavailable,
            Self::RenderFailed { .. } => ErrorCode::RenderFailed,
            Self::RenderRequestNotFound { .. } => ErrorCode::RenderRequestNotFound,
            Self::TemplateInvalid { .. } => ErrorCode::TemplateInvalid,
            Self::TemplateCheckNotFound { .. } => ErrorCode::TemplateCheckNotFound,
            Self::OutOfScope { .. } => ErrorCode::OutOfScope,
            Self::NothingToRevert { .. } => ErrorCode::NothingToRevert,
            Self::TranscriptionUnavailable { .. } => ErrorCode::TranscriptionUnavailable,
            Self::NoTranscriptionModel => ErrorCode::NoTranscriptionModel,
            Self::VisionUnavailable { .. } => ErrorCode::VisionUnavailable,
            Self::TranscriptionTargetNotFound { .. } => ErrorCode::TranscriptionTargetNotFound,
            Self::InvalidModelFile { .. } => ErrorCode::InvalidModelFile,
            Self::DownloadFailed { .. } => ErrorCode::DownloadFailed,
            Self::NoSpeechStream { .. } => ErrorCode::NoSpeechStream,
            Self::Subtitles(crate::ports::SubtitleError::Invalid { .. }) => {
                ErrorCode::InvalidSubtitleFile
            }
            Self::Subtitles(_) => ErrorCode::SubtitleFileFailed,
            Self::LibraryTemplateNotFound { .. } => ErrorCode::LibraryTemplateNotFound,
            Self::BrandKitNotFound { .. } => ErrorCode::BrandKitNotFound,
            Self::InvalidLibraryFile { .. } => ErrorCode::InvalidLibraryFile,
            Self::InvalidLibraryEntry { .. } => ErrorCode::InvalidBrandKit,
            Self::Internal(_) => ErrorCode::Internal,
        }
    }
}

/// The problems of a template, one per line: `line 12, column 3 (at 1.200 s): message`.
fn describe_template_problems(problems: &[TemplateProblem]) -> String {
    let lines: Vec<String> = problems
        .iter()
        .map(|problem| {
            let mut place = Vec::new();
            if let Some(line) = problem.line {
                place.push(match problem.column {
                    Some(column) => format!("line {line}, column {column}"),
                    None => format!("line {line}"),
                });
            }
            if let Some(time) = problem.time {
                place.push(format!("at {:.3} s", time.to_seconds_f64()));
            }
            let stage = match problem.stage {
                TemplateCheckStage::Compile => "compile",
                TemplateCheckStage::Exports => "exports",
                TemplateCheckStage::Render => "render",
                TemplateCheckStage::Budget => "budget",
                TemplateCheckStage::Determinism => "determinism",
                TemplateCheckStage::Definition => "definition",
            };
            if place.is_empty() {
                format!("- [{stage}] {}", problem.message)
            } else {
                format!("- [{stage}] {}: {}", place.join(", "), problem.message)
            }
        })
        .collect();
    format!(
        "the motion template cannot be used; fix these problems and send the whole module again:\n{}",
        lines.join("\n")
    )
}

/// The protocol code of a media port failure (also used per file in import reports).
#[must_use]
pub(crate) fn media_code(error: &MediaError) -> ErrorCode {
    match error {
        MediaError::NotFound { .. } => ErrorCode::MediaNotFound,
        MediaError::Unsupported { .. } => ErrorCode::MediaUnsupported,
        MediaError::ToolsMissing { .. } => ErrorCode::MediaToolsMissing,
        MediaError::Io { .. } => ErrorCode::StorageFailed,
        MediaError::Failed(_) => ErrorCode::Internal,
    }
}

fn storage_code(error: &StorageError) -> ErrorCode {
    match error {
        StorageError::NotFound { .. } => ErrorCode::ProjectNotFound,
        StorageError::Locked { .. } => ErrorCode::ProjectLocked,
        StorageError::AlreadyExists { .. } => ErrorCode::ProjectAlreadyExists,
        StorageError::SchemaTooNew { .. } => ErrorCode::SchemaTooNew,
        StorageError::Corrupted { .. } => ErrorCode::ProjectCorrupted,
        StorageError::Io { .. } => ErrorCode::StorageFailed,
    }
}

impl From<AppError> for ProtocolError {
    fn from(error: AppError) -> Self {
        let protocol = Self::new(error.code(), error.to_string());
        match &error {
            // `{"domainCode":"OVERLAPPING_ITEMS"}`: clients and agents can match the domain code.
            AppError::Domain(domain) => {
                protocol.with_detail("domainCode", domain.code().to_string())
            }
            _ => protocol,
        }
    }
}

#[cfg(test)]
mod tests {
    use se_domain::ItemId;

    use super::*;

    #[test]
    fn storage_errors_map_to_project_codes() {
        let locked = AppError::from(StorageError::Locked {
            path: PathBuf::from("Film.seproj"),
        });
        assert_eq!(locked.code(), ErrorCode::ProjectLocked);
        assert!(locked.to_string().contains("Film.seproj"));
    }

    #[test]
    fn domain_errors_carry_their_domain_code() {
        let error = AppError::from(DomainError::ItemNotFound {
            item: ItemId::from_test_index(9),
        });
        let protocol = ProtocolError::from(error);
        assert_eq!(protocol.code, ErrorCode::EditRejected);
        let details = protocol.details.unwrap();
        assert_eq!(details["domainCode"], "ITEM_NOT_FOUND");
    }

    #[test]
    fn nothing_to_undo_keeps_its_own_code() {
        assert_eq!(
            AppError::from(DomainError::NothingToUndo).code(),
            ErrorCode::NothingToUndo
        );
    }
}

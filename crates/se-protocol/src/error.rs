//! Protocol errors: a stable code plus an actionable message.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// An error answering a request. Messages are read by users and AI agents: keep them actionable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
pub struct ProtocolError {
    /// Stable, machine-readable error code.
    pub code: ErrorCode,
    /// Human- and agent-readable explanation, including how to fix the problem.
    pub message: String,
    /// Optional structured context (e.g. the unknown command kind). Omitted when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[ts(optional)]
    pub details: Option<serde_json::Value>,
}

impl ProtocolError {
    /// An error without details.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    /// The same error carrying structured `details`.
    #[must_use]
    pub fn with_details(self, details: serde_json::Value) -> Self {
        Self {
            details: Some(details),
            ..self
        }
    }

    /// The same error with the string field `key` added to its object `details` (created when
    /// absent; non-object details are replaced).
    #[must_use]
    pub fn with_detail(self, key: &str, value: impl Into<String>) -> Self {
        let mut fields = match self.details {
            Some(serde_json::Value::Object(fields)) => fields,
            _ => serde_json::Map::new(),
        };
        fields.insert(key.to_owned(), serde_json::Value::String(value.into()));
        Self {
            details: Some(serde_json::Value::Object(fields)),
            ..self
        }
    }
}

/// Stable error codes (`SCREAMING_SNAKE_CASE` on the wire). Never rename or reuse a code: clients
/// and AI agents match on them. Later prompts add codes for their own failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, TS)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    /// The message is not valid JSON, is not a request envelope, or the command's fields are wrong.
    InvalidRequest,
    /// The request is well-formed but this `se-core` does not know the command `kind`.
    UnknownCommand,
    /// Unexpected failure inside `se-core` (a bug): retry, and report it if it persists.
    Internal,
    /// No response arrived in time. Raised by clients (`CoreClient`), not sent by `se-core`.
    Timeout,
    /// The connection dropped after the request was sent: the command may or may not have run.
    /// Raised by clients, not sent by `se-core`.
    ConnectionLost,
    /// The client was closed by its owner before the response arrived. Raised by clients.
    ClientClosed,
    /// No project folder (with a `project.json`) at the given path.
    ProjectNotFound,
    /// The project is open in another SimpleEffect process (or another copy of it is open here).
    ProjectLocked,
    /// The project was saved by a newer SimpleEffect: update the app to open it.
    SchemaTooNew,
    /// The project's files cannot be read (invalid JSON, broken journal, invalid document).
    ProjectCorrupted,
    /// A project folder with that name already exists in the chosen location.
    ProjectAlreadyExists,
    /// The command names a project that is not open in this core: open it first.
    ProjectNotOpen,
    /// Reading or writing project or library files failed (disk full, permissions…).
    StorageFailed,
    /// The document refused the edit (overlap, locked track, stale revision…); `details.domainCode`
    /// holds the domain's code.
    EditRejected,
    /// There is no user transaction to undo.
    NothingToUndo,
    /// There is no undone transaction to redo.
    NothingToRedo,
    /// The project has no asset with that id.
    AssetNotFound,
    /// No file exists at the given media path.
    MediaNotFound,
    /// The file is not a video, audio or image file ffprobe can read.
    MediaUnsupported,
    /// The replacement file holds another kind of media than the asset (video vs audio vs image).
    MediaKindMismatch,
    /// ffmpeg or ffprobe could not be found: run `pnpm fetch-binaries` (development) or reinstall
    /// the app, or set the folder holding them.
    MediaToolsMissing,
    /// No job with that id is queued or running.
    JobNotFound,
    /// The derived copy does not apply to this asset (a proxy of a sound file, a mezzanine of
    /// media WebCodecs already decodes).
    DerivedNotApplicable,
    /// The requested time is outside the media (before 0 or after its end).
    MediaTimeOutOfRange,
    /// A ripple edit would cut content of a track it must shift (to keep linked items in sync):
    /// lock that track to leave it out, or move that content first.
    RippleBlocked,
    /// A cut has no media to cross-fade (no handle on either side, or the items are too short or
    /// already joined by a transition there).
    TransitionNoRoom,
    /// The project has no AI task with that id.
    TaskNotFound,
    /// The AI task is not queued or running (cancelling a finished task).
    TaskNotActive,
    /// The AI task is still queued or running: wait for it, or cancel it first.
    TaskStillActive,
    /// The AI task has no agent session to resume (it never started, or the provider cannot
    /// resume): retry it or submit a new task.
    TaskNotResumable,
    /// The AI provider cannot run tasks: its CLI is not installed or not logged in.
    AiUnavailable,
    /// No editor window could draw frames of the sequence (none is open, or it did not answer in
    /// time): open the project in SimpleEffect and try again.
    RendererUnavailable,
    /// The renderer could not draw the requested frames (its message says why).
    RenderFailed,
    /// No frame render with that id is waiting (it finished, failed or timed out).
    RenderRequestNotFound,
    /// An AI agent tried to change something outside its task's scope (time range, tracks or
    /// sequence): the message gives the scope, the faulty range and what to do.
    OutOfScope,
    /// The AI task has no edit left to revert (it changed nothing, or was reverted already).
    NothingToRevert,
    /// Transcription cannot run: whisper.cpp is missing or cannot start (the message says why).
    TranscriptionUnavailable,
    /// No transcription model is installed (or the chosen one is gone): download or import one.
    NoTranscriptionModel,
    /// No model, download or GPU pack matches what was asked.
    TranscriptionTargetNotFound,
    /// A file is not a whisper.cpp (ggml) model.
    InvalidModelFile,
    /// A download failed or did not match its expected SHA-256.
    DownloadFailed,
    /// The asset has no sound to transcribe.
    NoSpeechStream,
    /// A subtitle file could not be read or written (the message names it and why).
    SubtitleFileFailed,
    /// A file is not SRT (`.srt`); the message gives the line of the problem.
    InvalidSubtitleFile,
    /// A motion template module cannot be used: it does not compile, throws, is too slow, not
    /// deterministic or declares bad parameters (the message lists every problem with its line).
    TemplateInvalid,
    /// No template check with that id is waiting (it finished or timed out).
    TemplateCheckNotFound,
    /// No entry of the template library has that id (removed meanwhile?): list it again.
    LibraryTemplateNotFound,
    /// No brand kit of the library has that id (removed meanwhile?): list them again.
    BrandKitNotFound,
    /// A library file cannot be used: a `.setemplate` file that is not one, or a logo that is
    /// not an image (the message names the file and why).
    InvalidLibraryFile,
    /// A brand kit with unusable values (empty name or font, too many logos…; the message names
    /// each field).
    InvalidBrandKit,
    /// Faces cannot be detected on this machine (the ONNX Runtime library is missing, or the
    /// system has no build of it; the message says why): reframe by hand.
    VisionUnavailable,
    /// No export of this session has this id (it may have finished and been cleared).
    ExportNotFound,
    /// The export cannot be started as asked (settings, range or destination; the message says
    /// what to change).
    InvalidExport,
    /// The export is not being rendered (it is queued, finished or cancelled): uploads and
    /// reports are refused.
    ExportNotRendering,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn details_are_omitted_when_absent_and_kept_when_present() {
        let plain = ProtocolError::new(ErrorCode::Timeout, "late");
        assert_eq!(
            serde_json::to_string(&plain).unwrap(),
            r#"{"code":"TIMEOUT","message":"late"}"#
        );
        let detailed = ProtocolError::new(ErrorCode::UnknownCommand, "nope")
            .with_details(serde_json::json!({ "kind": "explode" }));
        let json = serde_json::to_string(&detailed).unwrap();
        assert_eq!(
            json,
            r#"{"code":"UNKNOWN_COMMAND","message":"nope","details":{"kind":"explode"}}"#
        );
        assert_eq!(
            serde_json::from_str::<ProtocolError>(&json).unwrap(),
            detailed
        );
    }

    #[test]
    fn with_detail_adds_string_fields() {
        let error = ProtocolError::new(ErrorCode::EditRejected, "no")
            .with_detail("domainCode", "TRACK_LOCKED")
            .with_detail("item", "x");
        assert_eq!(
            error.details,
            Some(serde_json::json!({ "domainCode": "TRACK_LOCKED", "item": "x" }))
        );
    }

    #[test]
    fn codes_are_screaming_snake_case() {
        assert_eq!(
            serde_json::to_string(&ErrorCode::ConnectionLost).unwrap(),
            r#""CONNECTION_LOST""#
        );
    }
}

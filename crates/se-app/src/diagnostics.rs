//! Diagnostics archives (prompt 30): « Exporter un diagnostic » writes a zip of the core's recent
//! logs and a report of the machine, AI CLI, tools and settings, masked by [`Redactor`] (home
//! folder → `~`, media and project names → hashes, secrets and e-mail addresses removed). It
//! never reads a project: no document, transcript, prompt or media goes in.

pub mod redaction;

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use se_protocol::{
    AiSettings, AiStatus, ClientDiagnostics, CoreInfo, DiagnosticsSummary, MediaSettings,
    MediaToolsStatus, TranscriptionStatus, VisionStatus,
};

pub use self::redaction::Redactor;
use crate::{
    AppError,
    ports::{ArchiveEntry, Clock, DiagnosticsFiles},
};

/// Log files put in an archive: the newest ones (one per day).
pub const MAX_LOG_FILES: usize = 7;
/// Each log file is cut to its last 2 MiB.
pub const MAX_LOG_BYTES: usize = 2 * 1024 * 1024;

/// What the core knows about the machine when the archive is written (gathered by se-core from
/// the services; every field is masked before it is written).
#[derive(Debug, Clone)]
pub struct DiagnosticsReport {
    /// The running core.
    pub core: CoreInfo,
    /// The AI CLI: found or not, logged in or not (never the account).
    pub ai_status: AiStatus,
    /// AI settings (model, effort, CLI path, tasks at once).
    pub ai_settings: AiSettings,
    /// ffmpeg and ffprobe.
    pub media_tools: MediaToolsStatus,
    /// Mezzanine and proxy settings.
    pub media_settings: MediaSettings,
    /// whisper.cpp, the GPU, installed models.
    pub transcription: TranscriptionStatus,
    /// Face detection.
    pub vision: VisionStatus,
    /// What the window adds (versions, GPU, codecs).
    pub client: ClientDiagnostics,
}

/// Writes diagnostics archives.
#[derive(Clone)]
pub struct DiagnosticsService {
    files: Arc<dyn DiagnosticsFiles>,
    clock: Arc<dyn Clock>,
    logs_dir: PathBuf,
    redactor: Redactor,
}

impl DiagnosticsService {
    /// A service reading logs in `logs_dir` and masking `home` (the user's home folder).
    pub fn new(
        files: Arc<dyn DiagnosticsFiles>,
        clock: Arc<dyn Clock>,
        logs_dir: PathBuf,
        home: Option<&Path>,
    ) -> Self {
        Self {
            files,
            clock,
            logs_dir,
            redactor: Redactor::new(home),
        }
    }

    /// Writes the archive of `report` and the recent logs to `path` (an absolute `.zip`).
    ///
    /// # Errors
    /// `InvalidSetting` for a relative path or another extension; storage errors when the logs
    /// cannot be read or the archive written.
    pub async fn export(
        &self,
        path: &Path,
        report: &DiagnosticsReport,
    ) -> Result<DiagnosticsSummary, AppError> {
        check_archive_path(path)?;
        let logs = self
            .files
            .recent_logs(&self.logs_dir, MAX_LOG_FILES, MAX_LOG_BYTES)
            .await?;
        let mut entries = vec![
            text_entry("README.txt", README),
            text_entry(
                "report.json",
                &self
                    .redactor
                    .redact(&report_json(report, self.clock.now_unix_millis())),
            ),
        ];
        entries.extend(logs.iter().map(|log| {
            text_entry(
                &format!("logs/{}", self.redactor.redact(&log.name)),
                &self.redactor.redact(&log.text),
            )
        }));
        let files = entries.iter().map(|entry| entry.name.clone()).collect();
        let bytes = self.files.write_archive(path, entries).await?;
        Ok(DiagnosticsSummary {
            path: path.display().to_string(),
            files,
            bytes,
        })
    }
}

/// What the archive holds, for whoever opens it.
const README: &str = "SimpleEffect diagnostics\n\
\n\
report.json  versions, operating system, AI CLI status (never the account), tools, settings,\n\
             graphics card and codecs\n\
logs/        the core's recent logs\n\
\n\
Masked: the home folder is written ~, media and project file names are replaced by hashes,\n\
tokens and e-mail addresses are removed. No project content (documents, transcripts, prompts,\n\
media) is included.\n";

fn text_entry(name: &str, text: &str) -> ArchiveEntry {
    ArchiveEntry {
        name: name.to_owned(),
        bytes: text.as_bytes().to_vec(),
    }
}

fn check_archive_path(path: &Path) -> Result<(), AppError> {
    let is_zip = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"));
    if path.is_absolute() && is_zip {
        Ok(())
    } else {
        Err(AppError::InvalidSetting {
            name: "diagnostics archive",
            reason: format!("{} must be an absolute path ending in .zip", path.display()),
        })
    }
}

/// The report as pretty JSON (before masking).
fn report_json(report: &DiagnosticsReport, written_at_ms: i64) -> String {
    let value = serde_json::json!({
        "writtenAtUnixMs": written_at_ms,
        "core": report.core,
        "aiStatus": report.ai_status,
        "aiSettings": report.ai_settings,
        "mediaTools": report.media_tools,
        "mediaSettings": report.media_settings,
        "transcription": report.transcription,
        "vision": report.vision,
        "client": report.client,
    });
    serde_json::to_string_pretty(&value)
        .unwrap_or_else(|error| format!("{{\"error\":\"{error}\"}}"))
}

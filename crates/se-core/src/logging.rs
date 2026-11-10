//! Tracing setup: daily log files in `<app data>/logs/`, plus stderr in development.
//!
//! stdout is reserved for the handshake line, so nothing here ever writes to it.

use std::{
    fs,
    io::{self, IsTerminal},
    path::Path,
};

use tracing_appender::{
    non_blocking::{NonBlocking, WorkerGuard},
    rolling::{InitError, RollingFileAppender, Rotation},
};
use tracing_subscriber::{EnvFilter, Layer, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Environment variable holding the log filter, e.g. `SE_LOG=se_core=debug,info`.
pub const LOG_FILTER_ENV: &str = "SE_LOG";

/// Filter used when `SE_LOG` is unset or invalid.
const DEFAULT_FILTER: &str = "info";

/// Log files are named `se-core.<YYYY-MM-DD>.log`.
const LOG_FILE_PREFIX: &str = "se-core";
const LOG_FILE_SUFFIX: &str = "log";

/// One file per day, so this keeps about a week of logs.
const RETAINED_LOG_FILES: usize = 7;

/// Keeps the background log writer alive: dropping it flushes pending lines to the file.
/// Hold it until the end of `main`.
#[must_use = "dropping the guard stops writing logs to the file"]
pub struct LogGuard {
    _file_writer: Option<WorkerGuard>,
}

/// Installs the global tracing subscriber.
///
/// Logs go to a daily file in `logs_dir` (7 files kept) and, in debug builds, to stderr as well
/// (forwarded to the Electron console by the supervisor). If the log folder is unusable, logs fall
/// back to stderr so they are never lost silently.
pub fn init(logs_dir: &Path) -> LogGuard {
    let file = open_log_file(logs_dir);
    let show_stderr = cfg!(debug_assertions) || file.is_err();
    let (file_writer, file_guard, file_error) = match file {
        Ok((writer, guard)) => (Some(writer), Some(guard), None),
        Err(error) => (None, None, Some(error)),
    };
    // One filter per output, so the tokio-console layer (development) sees tokio's own spans.
    let file_layer = file_writer.map(|writer| {
        fmt::layer()
            .with_writer(writer)
            .with_ansi(false)
            .with_filter(log_filter())
    });
    // ANSI colours only on a terminal, so the Electron supervisor receives plain text.
    let stderr_layer = show_stderr.then(|| {
        fmt::layer()
            .with_writer(io::stderr)
            .with_ansi(io::stderr().is_terminal())
            .with_filter(log_filter())
    });
    tracing_subscriber::registry()
        .with(console_layer())
        .with(file_layer)
        .with(stderr_layer)
        .init();
    if let Some(error) = file_error {
        tracing::warn!(
            %error,
            logs_dir = %logs_dir.display(),
            "cannot write log files; logging to stderr only"
        );
    }
    LogGuard {
        _file_writer: file_guard,
    }
}

/// The filter of `SE_LOG`, or the default one.
fn log_filter() -> EnvFilter {
    EnvFilter::try_from_env(LOG_FILTER_ENV).unwrap_or_else(|_| EnvFilter::new(DEFAULT_FILTER))
}

/// The tokio-console layer of development builds made with the `tokio-console` feature and
/// `RUSTFLAGS="--cfg tokio_unstable"` (`docs/performance.md`); none otherwise.
#[cfg(feature = "tokio-console")]
fn console_layer() -> Option<impl Layer<tracing_subscriber::Registry>> {
    Some(console_subscriber::spawn())
}

/// No tokio-console layer: the feature is off (always in shipped builds).
#[cfg(not(feature = "tokio-console"))]
fn console_layer() -> Option<tracing_subscriber::layer::Identity> {
    None
}

fn open_log_file(logs_dir: &Path) -> Result<(NonBlocking, WorkerGuard), InitError> {
    // Created up front: the appender prunes old files before creating the folder, and prints a
    // spurious error on a fresh install otherwise. If this fails, `build` reports the real error.
    let _ = fs::create_dir_all(logs_dir);
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix(LOG_FILE_PREFIX)
        .filename_suffix(LOG_FILE_SUFFIX)
        .max_log_files(RETAINED_LOG_FILES)
        .build(logs_dir)?;
    Ok(tracing_appender::non_blocking(appender))
}

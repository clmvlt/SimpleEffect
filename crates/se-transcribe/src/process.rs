//! Running whisper-cli (and the small helpers around it): the only place this crate spawns
//! processes (`docs/architecture.md` §14). Every process is isolated (job object / process group,
//! no console window), has no stdin, and dies with its tree when its future is dropped, which is
//! how a transcription job is cancelled.

use std::{ffi::OsStr, path::Path, process::Stdio, time::Duration};

use process_wrap::tokio::CommandWrap;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};

use crate::platform;

/// How a process ended.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finished {
    /// Whether it exited with success.
    pub success: bool,
    /// Its exit code, when it has one.
    pub code: Option<i32>,
    /// Everything it wrote to stderr.
    pub log: String,
}

/// Runs `program` with `args` in `cwd`, handing each stderr line to `on_line` (stdout is
/// drained and ignored).
///
/// # Errors
/// A message when the process cannot start or its pipes fail.
pub async fn run_logged<S: AsRef<OsStr>>(
    program: &Path,
    args: &[S],
    cwd: Option<&Path>,
    on_line: &mut (dyn FnMut(&str) + Send),
) -> Result<Finished, String> {
    let mut command = CommandWrap::with_new(program, |command| {
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if let Some(dir) = cwd {
            command.current_dir(dir);
        }
    });
    platform::isolate(&mut command);
    let mut child = command
        .spawn()
        .map_err(|error| format!("{} could not start: {error}", program.display()))?;
    let stdout = child.stdout().take();
    let drain = tokio::spawn(async move {
        if let Some(mut stdout) = stdout {
            let mut sink = Vec::new();
            // Its segment lines are not needed (the JSON file holds everything).
            let _ = stdout.read_to_end(&mut sink).await;
        }
    });
    let mut log = String::new();
    if let Some(stderr) = child.stderr().take() {
        let mut lines = BufReader::new(stderr).lines();
        while let Ok(Some(line)) = lines.next_line().await {
            on_line(&line);
            log.push_str(&line);
            log.push('\n');
        }
    }
    let status = child
        .wait()
        .await
        .map_err(|error| format!("waiting for {} failed: {error}", program.display()))?;
    drain.abort();
    Ok(Finished {
        success: status.success(),
        code: status.code(),
        log,
    })
}

/// The first line `program args` prints within five seconds, or `None` (absent program, error,
/// timeout).
pub async fn first_line(program: &Path, args: &[&str]) -> Option<String> {
    let mut command = CommandWrap::with_new(program, |command| {
        command
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
    });
    platform::isolate(&mut command);
    let mut child = command.spawn().ok()?;
    let mut stdout = child.stdout().take()?;
    let read = async {
        let mut text = String::new();
        stdout.read_to_string(&mut text).await.ok()?;
        let status = child.wait().await.ok()?;
        status
            .success()
            .then(|| text.lines().next().map(str::trim).map(str::to_owned))
            .flatten()
            .filter(|line| !line.is_empty())
    };
    tokio::time::timeout(Duration::from_secs(5), read)
        .await
        .ok()
        .flatten()
}

/// The last `count` lines of `log`, for error messages.
#[must_use]
pub fn tail(log: &str, count: usize) -> String {
    let lines: Vec<&str> = log.lines().filter(|line| !line.trim().is_empty()).collect();
    lines[lines.len().saturating_sub(count)..].join(" | ")
}

#[cfg(test)]
mod tests {
    use super::tail;

    #[test]
    fn tails_keep_the_last_non_empty_lines() {
        assert_eq!(tail("a\n\nb\nc\n", 2), "b | c");
        assert_eq!(tail("", 3), "");
    }
}

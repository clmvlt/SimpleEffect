//! [`Sidecar`]: the single helper that spawns ffmpeg and ffprobe (`docs/architecture.md` §14).
//!
//! Every process is isolated (job object on Windows, process group on Unix, no console window),
//! has no stdin, pipes stdout and stderr, and is killed with its whole tree when the [`Sidecar`]
//! is dropped before it exits, which is how cancelling a job (dropping its future) stops it.
//! stderr is collected in the background so error messages can quote its last lines.

use std::{ffi::OsStr, path::Path, process::Stdio};

use process_wrap::tokio::{ChildWrapper, CommandWrap};
use se_app::ports::MediaError;
use tokio::{
    io::{AsyncBufReadExt, AsyncReadExt, BufReader},
    process::ChildStdout,
    task::JoinHandle,
};

use crate::platform;

/// Characters of stderr kept for error messages (its end).
const STDERR_TAIL: usize = 2000;

/// A running sidecar process.
pub struct Sidecar {
    name: String,
    child: Box<dyn ChildWrapper>,
    stderr: Option<JoinHandle<String>>,
    exited: bool,
}

impl Sidecar {
    /// Starts `program` with `args`, in `cwd` when given.
    ///
    /// # Errors
    /// [`MediaError::Failed`] when the process cannot start.
    pub fn spawn<I, S>(program: &Path, args: I, cwd: Option<&Path>) -> Result<Self, MediaError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let name = program.file_stem().map_or_else(
            || "tool".to_owned(),
            |stem| stem.to_string_lossy().into_owned(),
        );
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
            .map_err(|error| MediaError::Failed(format!("{name} could not start: {error}")))?;
        let stderr = child.stderr().take().map(|stderr| {
            tokio::spawn(async move {
                let mut text = String::new();
                // A read error only shortens the message.
                let _ = BufReader::new(stderr).read_to_string(&mut text).await;
                text
            })
        });
        Ok(Self {
            name,
            child,
            stderr,
            exited: false,
        })
    }

    /// Takes the process's stdout (once).
    pub fn take_stdout(&mut self) -> Option<ChildStdout> {
        self.child.stdout().take()
    }

    /// Waits for the process to exit.
    ///
    /// # Errors
    /// [`MediaError::Failed`] with its exit status and the end of its stderr when it fails.
    pub async fn finish(mut self) -> Result<(), MediaError> {
        let status = self.child.wait().await;
        self.exited = true;
        let stderr = match self.stderr.take() {
            Some(task) => task.await.unwrap_or_default(),
            None => String::new(),
        };
        match status {
            Ok(status) if status.success() => Ok(()),
            Ok(status) => Err(MediaError::Failed(format!(
                "{} failed ({status}): {}",
                self.name,
                tail(&stderr)
            ))),
            Err(error) => Err(MediaError::Failed(format!(
                "waiting for {} failed: {error}",
                self.name
            ))),
        }
    }

    /// Runs the process to completion and returns its stdout.
    ///
    /// # Errors
    /// See [`Sidecar::finish`].
    pub async fn output(mut self) -> Result<Vec<u8>, MediaError> {
        let mut stdout = Vec::new();
        if let Some(mut pipe) = self.take_stdout() {
            pipe.read_to_end(&mut stdout).await.map_err(|error| {
                MediaError::Failed(format!("reading {} failed: {error}", self.name))
            })?;
        }
        self.finish().await?;
        Ok(stdout)
    }

    /// Runs the process to completion, handing every stdout line to `on_line` (ffmpeg
    /// `-progress pipe:1` output).
    ///
    /// # Errors
    /// See [`Sidecar::finish`].
    pub async fn run_with_lines(mut self, mut on_line: impl FnMut(&str)) -> Result<(), MediaError> {
        if let Some(pipe) = self.take_stdout() {
            let mut lines = BufReader::new(pipe).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                on_line(&line);
            }
        }
        self.finish().await
    }
}

impl Drop for Sidecar {
    fn drop(&mut self) {
        if !self.exited {
            // Cancelled before it exited: stop the whole tree (job or process group).
            if let Err(error) = self.child.start_kill() {
                tracing::warn!(%error, tool = %self.name, "stopping a cancelled sidecar failed");
            }
        }
    }
}

/// The last characters of `text`, trimmed, for error messages.
fn tail(text: &str) -> String {
    let trimmed = text.trim();
    let start = trimmed
        .char_indices()
        .rev()
        .nth(STDERR_TAIL)
        .map_or(0, |(index, _)| index);
    let tail = &trimmed[start..];
    if tail.is_empty() {
        "no error output".to_owned()
    } else {
        tail.to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_output_is_trimmed_to_its_end() {
        assert_eq!(tail("  \n"), "no error output");
        let long = format!("{}end", "x".repeat(5000));
        let kept = tail(&long);
        assert!(kept.ends_with("end"));
        assert!(kept.len() <= STDERR_TAIL + 1);
    }
}

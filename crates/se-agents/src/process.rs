//! [`CliProcess`]: the single helper that spawns AI CLIs (`docs/architecture.md` §14).
//!
//! Every process is isolated (`platform::isolate`), gets its input on stdin (then end-of-file),
//! pipes stdout and stderr, and is killed with its whole tree when stopped or dropped before it
//! exits. stderr is collected in the background so failures can quote its end.

use std::{
    ffi::{OsStr, OsString},
    path::Path,
    process::{ExitStatus, Stdio},
    time::Duration,
};

use process_wrap::tokio::{ChildWrapper, CommandWrap};
use se_app::ports::AgentError;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader, Lines},
    process::ChildStdout,
    task::JoinHandle,
};

use crate::platform;

/// Characters of stderr kept for error messages (its end).
const STDERR_TAIL: usize = 2000;

/// A running CLI process.
pub struct CliProcess {
    name: String,
    child: Box<dyn ChildWrapper>,
    stderr: Option<JoinHandle<String>>,
    exited: bool,
}

/// What a short CLI command printed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandOutput {
    /// Whether it exited with success.
    pub success: bool,
    /// Its exit status, for messages.
    pub status: String,
    /// What it wrote on stdout.
    pub stdout: String,
    /// The end of what it wrote on stderr.
    pub stderr: String,
}

impl CliProcess {
    /// Starts `program` with `args` in `cwd`, without the environment variables `unset`, writing
    /// `input` to its stdin then closing it.
    ///
    /// # Errors
    /// [`AgentError::Failed`] when the process cannot start.
    pub fn spawn<I, S>(
        program: &Path,
        args: I,
        cwd: Option<&Path>,
        input: Option<String>,
        unset: &[OsString],
    ) -> Result<Self, AgentError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let name = program.file_stem().map_or_else(
            || "cli".to_owned(),
            |stem| stem.to_string_lossy().into_owned(),
        );
        let mut command = CommandWrap::with_new(program, |command| {
            command
                .args(args)
                .stdin(if input.is_some() {
                    Stdio::piped()
                } else {
                    Stdio::null()
                })
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            if let Some(dir) = cwd {
                command.current_dir(dir);
            }
            for name in unset {
                command.env_remove(name);
            }
        });
        platform::isolate(&mut command);
        let mut child = command.spawn().map_err(|error| AgentError::Failed {
            message: format!("{} could not start: {error}", program.display()),
        })?;
        if let (Some(input), Some(mut stdin)) = (input, child.stdin().take()) {
            tokio::spawn(async move {
                // A CLI that exits early closes its stdin: its own error says why.
                let _ = stdin.write_all(input.as_bytes()).await;
                let _ = stdin.shutdown().await;
            });
        }
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

    /// The process's stdout, line by line (once).
    pub fn stdout_lines(&mut self) -> Option<Lines<BufReader<ChildStdout>>> {
        use tokio::io::AsyncBufReadExt;
        self.child
            .stdout()
            .take()
            .map(|stdout| BufReader::new(stdout).lines())
    }

    /// Waits for the process to exit; returns its status and the end of its stderr.
    ///
    /// # Errors
    /// [`AgentError::Failed`] when waiting fails.
    pub async fn wait(&mut self) -> Result<(ExitStatus, String), AgentError> {
        let status = self
            .child
            .wait()
            .await
            .map_err(|error| AgentError::Failed {
                message: format!("waiting for {} failed: {error}", self.name),
            })?;
        self.exited = true;
        let stderr = match self.stderr.take() {
            Some(task) => task.await.unwrap_or_default(),
            None => String::new(),
        };
        Ok((status, tail(&stderr)))
    }

    /// Kills the whole process tree and waits for it to be gone.
    pub async fn kill(&mut self) {
        if self.exited {
            return;
        }
        if let Err(error) = self.child.start_kill() {
            tracing::warn!(%error, cli = %self.name, "stopping a CLI failed");
        }
        // Reaps the process; its status does not matter any more.
        let _ = self.wait().await;
    }
}

impl Drop for CliProcess {
    fn drop(&mut self) {
        if !self.exited
            && let Err(error) = self.child.start_kill()
        {
            tracing::warn!(%error, cli = %self.name, "stopping a dropped CLI failed");
        }
    }
}

/// Runs a short command (`--version`, `auth status`) to completion within `timeout`, without
/// the environment variables `unset`.
///
/// # Errors
/// [`AgentError::Failed`] when it cannot start or fails to exit in time.
pub async fn run_command(
    program: &Path,
    args: &[&str],
    timeout: Duration,
    unset: &[OsString],
) -> Result<CommandOutput, AgentError> {
    let mut process = CliProcess::spawn(program, args, None, None, unset)?;
    let mut stdout = Vec::new();
    let run = async {
        if let Some(mut pipe) = process.child.stdout().take() {
            pipe.read_to_end(&mut stdout)
                .await
                .map_err(|error| AgentError::Failed {
                    message: format!("reading {} failed: {error}", program.display()),
                })?;
        }
        process.wait().await
    };
    let (status, stderr) =
        tokio::time::timeout(timeout, run)
            .await
            .map_err(|_| AgentError::Failed {
                message: format!(
                    "{} {} did not answer within {} s",
                    program.display(),
                    args.join(" "),
                    timeout.as_secs()
                ),
            })??;
    Ok(CommandOutput {
        success: status.success(),
        status: status.to_string(),
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr,
    })
}

/// The last characters of `text`, trimmed.
#[must_use]
pub fn tail(text: &str) -> String {
    let trimmed = text.trim();
    let start = trimmed
        .char_indices()
        .rev()
        .nth(STDERR_TAIL)
        .map_or(0, |(index, _)| index);
    trimmed[start..].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_output_is_trimmed_to_its_end() {
        assert_eq!(tail("  \n"), "");
        let long = format!("{}end", "x".repeat(5000));
        let kept = tail(&long);
        assert!(kept.ends_with("end"));
        assert!(kept.len() <= STDERR_TAIL + 1);
    }
}

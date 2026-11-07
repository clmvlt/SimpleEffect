//! One Claude Code run: the process, its stdout parsed line by line into events, cancellation
//! (the whole process tree), and the terminal event when the CLI ends without a result.

use std::path::Path;

use se_app::ports::{AgentError, AgentRun, AgentRunSpec, CancelHandle, CancelSignal};
use se_protocol::{AgentEvent, AgentFailureKind};
use tokio::sync::mpsc;

use super::{
    arguments::{INSTRUCTIONS_FILE, redacted, run_arguments},
    stream::{StreamParser, missing_result},
};
use crate::process::CliProcess;

/// Events buffered between the CLI and the task service.
const EVENT_BUFFER: usize = 256;

/// Writes the instructions file, starts `claude` in the run's folder and returns its events.
///
/// # Errors
/// [`AgentError::Failed`] when the instructions cannot be written or the process cannot start.
pub(super) async fn start(executable: &Path, spec: AgentRunSpec) -> Result<AgentRun, AgentError> {
    let instructions = spec.working_dir.join(INSTRUCTIONS_FILE);
    tokio::fs::write(&instructions, &spec.system_instructions)
        .await
        .map_err(|error| AgentError::Failed {
            message: format!(
                "writing the agent's instructions to {} failed: {error}",
                instructions.display()
            ),
        })?;
    let args = run_arguments(&spec, &instructions);
    tracing::info!(executable = %executable.display(), args = ?redacted(&args), "starting Claude Code");
    let process = CliProcess::spawn(
        executable,
        &args,
        Some(&spec.working_dir),
        Some(spec.prompt),
    )?;
    let (sender, events) = mpsc::channel(EVENT_BUFFER);
    let (cancel, signal) = CancelHandle::new();
    tokio::spawn(follow(process, sender, signal));
    Ok(AgentRun { events, cancel })
}

/// Forwards the run's events until its process is gone, stopping it when cancelled or when
/// nobody listens any more.
async fn follow(
    mut process: CliProcess,
    sender: mpsc::Sender<AgentEvent>,
    mut signal: CancelSignal,
) {
    let mut parser = StreamParser::default();
    let mut ended = false;
    if let Some(mut lines) = process.stdout_lines() {
        loop {
            let line = tokio::select! {
                biased;
                () = signal.cancelled() => {
                    stop(&mut process, &sender, ended, "the task was cancelled").await;
                    return;
                }
                line = lines.next_line() => line,
            };
            let Ok(Some(line)) = line else { break };
            for event in parser.parse_line(&line) {
                ended |= event.is_terminal();
                if sender.send(event).await.is_err() {
                    process.kill().await;
                    return;
                }
            }
        }
    }
    let exit = tokio::select! {
        biased;
        () = signal.cancelled() => {
            stop(&mut process, &sender, ended, "the task was cancelled").await;
            return;
        }
        exit = process.wait() => exit,
    };
    if ended {
        return;
    }
    let event = match exit {
        Ok((status, stderr)) => {
            missing_result(&parser, status.success(), &status.to_string(), &stderr)
        }
        Err(error) => error.to_event(),
    };
    // Nobody listening any more is fine: the process is gone.
    let _ = sender.send(event).await;
}

/// Kills the process tree and reports the run cancelled (unless it had already ended).
async fn stop(
    process: &mut CliProcess,
    sender: &mpsc::Sender<AgentEvent>,
    ended: bool,
    message: &str,
) {
    process.kill().await;
    if !ended {
        let _ = sender
            .send(AgentEvent::Failed {
                failure: AgentFailureKind::Cancelled,
                message: message.to_owned(),
            })
            .await;
    }
}

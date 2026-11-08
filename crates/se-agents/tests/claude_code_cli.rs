//! `ClaudeCodeProvider` against a fake `claude` (a shell script replaying a recorded stream-json
//! fixture): version and login detection, the command line, the prompt on stdin, the events,
//! failures without a result, and cancellation killing the whole process tree. Unix only: the
//! fake is a POSIX shell script.

#![cfg(unix)]
#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    time::Duration,
};

use se_agents::providers::claude_code::ClaudeCodeProvider;
use se_app::ports::{AgentProvider, AgentRun, AgentRunSpec, AgentSession};
use se_protocol::{AgentEvent, AgentFailureKind, AuthStatus};
use tempfile::TempDir;

const FAKE_CLAUDE: &str = r#"#!/bin/sh
dir=$(dirname "$0")
case "$1" in
  --version) echo "9.9.9 (Claude Code)"; exit 0 ;;
  auth) cat "$dir/auth.json"; exit 0 ;;
esac
printf '%s\n' "$@" > "$dir/args.txt"
cat > "$dir/stdin.txt"
cat "$dir/stream.jsonl"
if [ -f "$dir/stderr.txt" ]; then cat "$dir/stderr.txt" >&2; exit 1; fi
if [ -f "$dir/hang" ]; then sleep 60 & echo $! > "$dir/child.pid"; wait; fi
"#;

struct Fake {
    bin: TempDir,
    work: TempDir,
    provider: ClaudeCodeProvider,
}

fn fixture(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/claude_code")
        .join(name);
    std::fs::read_to_string(path).unwrap()
}

fn fake(stream: &str) -> Fake {
    let bin = tempfile::tempdir().unwrap();
    let script = bin.path().join("claude");
    std::fs::write(&script, FAKE_CLAUDE).unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    std::fs::write(bin.path().join("stream.jsonl"), stream).unwrap();
    std::fs::write(
        bin.path().join("auth.json"),
        r#"{"loggedIn":true,"authMethod":"claude.ai","subscriptionType":"max","email":"someone@example.com"}"#,
    )
    .unwrap();
    let provider = ClaudeCodeProvider::new();
    provider.set_executable(Some(script));
    Fake {
        bin,
        work: tempfile::tempdir().unwrap(),
        provider,
    }
}

impl Fake {
    fn spec(&self, session: AgentSession) -> AgentRunSpec {
        AgentRunSpec {
            prompt: "Say hello\nin one word.".to_owned(),
            system_instructions: "# Role\nYou edit videos.".to_owned(),
            mcp: None,
            model: Some("haiku".to_owned()),
            effort: None,
            session,
            working_dir: self.work.path().to_path_buf(),
        }
    }

    fn file(&self, name: &str) -> PathBuf {
        self.bin.path().join(name)
    }
}

async fn collect(mut run: AgentRun) -> Vec<AgentEvent> {
    let mut events = Vec::new();
    while let Some(event) = tokio::time::timeout(Duration::from_secs(10), run.events.recv())
        .await
        .expect("the run must end")
    {
        events.push(event);
    }
    events
}

#[tokio::test]
async fn detection_reads_the_version_and_the_login_state() {
    let fake = fake("");
    let installation = fake.provider.detect().await.unwrap();
    assert_eq!(installation.version, "9.9.9");
    assert_eq!(installation.path, fake.file("claude"));
    assert_eq!(
        fake.provider.auth_status().await.unwrap(),
        AuthStatus::LoggedIn {
            account_kind: Some("max".to_owned())
        }
    );
}

#[tokio::test]
async fn a_run_gets_its_prompt_on_stdin_and_streams_the_cli_events() {
    let fake = fake(&fixture("not-logged-in.jsonl"));
    let session = "0195f3a2-7c41-7d2e-9a10-3b4c5d6e7f81";
    let run = fake
        .provider
        .start(fake.spec(AgentSession::Start(session.to_owned())))
        .await
        .unwrap();
    let events = collect(run).await;
    assert_eq!(
        events.first(),
        Some(&AgentEvent::SessionStarted {
            session_id: session.to_owned(),
            model: Some("claude-opus-5-5".to_owned())
        })
    );
    let Some(AgentEvent::Failed { failure, message }) = events.last() else {
        panic!("expected a failure, got {events:?}");
    };
    assert_eq!(*failure, AgentFailureKind::NotAuthenticated);
    assert!(message.contains("/login"));
    assert_eq!(
        std::fs::read_to_string(fake.file("stdin.txt")).unwrap(),
        "Say hello\nin one word."
    );
    let args = std::fs::read_to_string(fake.file("args.txt")).unwrap();
    let args: Vec<&str> = args.lines().collect();
    assert!(
        args.windows(2)
            .any(|pair| pair == ["--session-id", session])
    );
    assert!(args.windows(2).any(|pair| pair == ["--tools", ""]));
    assert!(args.windows(2).any(|pair| pair == ["--model", "haiku"]));
    let instructions = fake.work.path().join(".simpleeffect-instructions.md");
    assert_eq!(
        std::fs::read_to_string(instructions).unwrap(),
        "# Role\nYou edit videos."
    );
}

#[tokio::test]
async fn a_recorded_success_completes_through_the_process() {
    let fake = fake(&fixture("success.jsonl"));
    let session = "0195f3a2-7c41-7d2e-9a10-3b4c5d6e7f91";
    let run = fake
        .provider
        .start(fake.spec(AgentSession::Start(session.to_owned())))
        .await
        .unwrap();
    let events = collect(run).await;
    assert!(matches!(
        events.first(),
        Some(AgentEvent::SessionStarted { session_id, .. }) if session_id == session
    ));
    assert_eq!(
        events.last(),
        Some(&AgentEvent::Completed {
            summary: "Bonjour".to_owned()
        })
    );
}

#[tokio::test]
async fn a_cli_ending_without_a_result_is_explained_from_its_stderr() {
    let fake = fake("");
    std::fs::write(
        fake.file("stderr.txt"),
        "No conversation found with session ID: 0195f3a2-7c41-7d2e-9a10-3b4c5d6e7f81",
    )
    .unwrap();
    let run = fake
        .provider
        .start(fake.spec(AgentSession::Resume("0195f3a2".to_owned())))
        .await
        .unwrap();
    let events = collect(run).await;
    let [AgentEvent::Failed { failure, message }] = events.as_slice() else {
        panic!("expected one failure, got {events:?}");
    };
    assert_eq!(*failure, AgentFailureKind::Crashed);
    assert!(message.contains("no longer exists"), "{message}");
    let args = std::fs::read_to_string(fake.file("args.txt")).unwrap();
    assert!(args.contains("--resume\n0195f3a2\n"));
}

#[tokio::test]
async fn cancelling_a_run_kills_the_whole_process_tree() {
    let init = fixture("not-logged-in.jsonl")
        .lines()
        .next()
        .unwrap()
        .to_owned();
    let fake = fake(&format!("{init}\n"));
    std::fs::write(fake.file("hang"), "").unwrap();
    let mut run = fake
        .provider
        .start(fake.spec(AgentSession::Start("s".to_owned())))
        .await
        .unwrap();
    let first = run.events.recv().await.unwrap();
    assert!(matches!(first, AgentEvent::SessionStarted { .. }));
    let pid_file = fake.file("child.pid");
    while !pid_file.exists() {
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let child = std::fs::read_to_string(&pid_file)
        .unwrap()
        .trim()
        .to_owned();
    run.cancel.cancel();
    let rest = collect(run).await;
    assert!(matches!(
        rest.as_slice(),
        [AgentEvent::Failed {
            failure: AgentFailureKind::Cancelled,
            ..
        }]
    ));
    let mut alive = true;
    for _ in 0..100 {
        let probe = std::process::Command::new("kill")
            .args(["-0", &child])
            .output()
            .unwrap();
        alive = probe.status.success();
        if !alive {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(
        !alive,
        "the CLI's child process {child} must be killed with it"
    );
}

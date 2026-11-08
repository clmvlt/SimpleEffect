# se-agents

## Purpose
AI CLI providers and their stream parsing (ADR-0003, ADR-0027). Claude Code is the first provider. This crate finds the CLI, checks its version and login, launches runs in empty working folders, maps the CLI's streaming output to `AgentEvent`s and cancels runs (whole process tree). Scheduling, persistence and settings belong to se-app's `TaskService`; agents act on the project only through MCP tools (`se-mcp`, prompts 18–19): every run gets se-core's `/mcp` URL and its own token in `AgentRunSpec.mcp`.

## Public API
| Item | Role |
|---|---|
| `providers::claude_code::ClaudeCodeProvider` | `AgentProvider` for Claude Code (`new()`, `PROVIDER_ID` = `claude-code`); flags, isolation and empirical findings in `src/providers/claude_code/CLAUDE.md` |
| `FsAgentWorkspaces::new(root)` | `AgentWorkspaces`: `<root>/<task id>/` emptied at `prepare`, removed at `release`; `folder(task_id)`; `remove_leftovers()` at startup (se-core calls it) |
| `process::CliProcess` | The only way to spawn a CLI: isolated (`platform::isolate`), variables to unset removed, input written to stdin then closed, stdout read by lines (`stdout_lines`), stderr tail collected, `wait`, `kill` (tree), killed on drop |
| `process::run_command(program, args, timeout)` | Short commands (`--version`, `auth status`) → `CommandOutput { success, status, stdout, stderr }` |

## Layout
| File | Role |
|---|---|
| `platform.rs` | OS specifics of processes: job object + `CREATE_NO_WINDOW` (Windows) / process group (Unix), kill on drop |
| `process.rs` | `CliProcess`, `run_command`, `tail` |
| `workspaces.rs` | `FsAgentWorkspaces` |
| `providers.rs`, `providers/claude_code.rs` | The provider (trait impl: `info`, `capabilities`, `set_executable`, `detect`, `auth_status`, `start`) |
| `providers/claude_code/identity.rs` | `ProviderInfo` (name, `claude`, `/login`, help URL) and capabilities (resume, images, model aliases, effort levels) |
| `providers/claude_code/locate.rs`, `platform.rs` | Executable resolution: setting → PATH (`which`) → usual install folders per OS |
| `providers/claude_code/arguments.rs` | Pure: the command line of a run, `redacted` (MCP config hidden in logs) |
| `providers/claude_code/run.rs` | A run: instructions file, process, events forwarded, cancellation, terminal event when the CLI ends without a result |
| `providers/claude_code/stream.rs` | `StreamParser` (stream-json → `AgentEvent`) and `missing_result` |
| `providers/claude_code/environment.rs` | Variables of a parent Claude Code session removed from the CLI's environment (only when `CLAUDECODE` is set; user configuration kept) |
| `providers/claude_code/auth.rs` | `parse_version`, `parse_auth_status` |
| `providers/claude_code/failure.rs` | `classify` (rate limit, login, misconfigured — a wrapper adding options —, crash) and `explain` (actionable messages) |

## Rules and invariants
- The **only** crate that knows which AI CLI is used (golden rule 3), and only `src/providers/<provider>/` knows a given CLI.
- Agents are launched with no built-in tools (`--tools ""` for Claude Code), in an empty working folder, isolated from the user's own CLI configuration (ADR-0027).
- Text with line breaks (prompts, instructions) never goes on the command line: Windows `.cmd` shims cannot receive it. Prompts go to stdin, instructions to a file in the working folder.
- Resolve executables with the `which` crate (PATHEXT-aware) after the user's override, then the usual install folders (desktop apps lack the shell's PATH). On Windows, a real `.exe` on the PATH wins over script wrappers (`.cmd`/`.bat`), which may add options of their own.
- Every process goes through `CliProcess`: no console window, the whole tree killed on cancel and on drop. A run ends with exactly one terminal event, then its channel closes once the process is gone.
- Unknown stream lines are logged at debug level and ignored (forward compatible); unreadable ones are counted and make a result-less end a `protocolError`.
- Never log or persist CLI credentials (`auth status` is parsed for `loggedIn` and the account kind only); redact MCP task tokens (`arguments::redacted`).

## Allowed dependencies
`se-app`, `se-domain`, `se-protocol` (DTOs: `AgentEvent`, `AuthStatus`, `ProviderInfo`…), `tokio` (process, io, fs), `process-wrap`, `which`, `directories` (home folder), `serde_json`, `async-trait`, `tracing`, `windows` (creation flags, Windows only). Never another adapter crate. See `docs/architecture.md` §3 and §9.

## How to test
- `cargo test -p se-agents`
- Unit tests in-file: recorded runs replayed through the parser (`stream.rs`: `success`, `resume`, `long-text`, `lost-session`, `not-logged-in`, `mcp-get-timeline` — a run calling the MCP tool `get_timeline` — fixtures), arguments (session flags, no line breaks, MCP config and its redaction), stream parsing (tool call/result pairing, session announced once, error results, malformed and unknown lines, result-less ends from stderr), auth and version parsing, failure classification, executable resolution, workspaces.
- `tests/claude_code_cli.rs` (Unix): a fake `claude` shell script replaying `tests/fixtures/claude_code/*.jsonl` — detection, prompt on stdin, arguments (MCP config and allowed tools always present), instructions file, an MCP tool call streamed through the process, a CLI ending with stderr only, cancellation killing the script's child process.
- Fixtures are recorded from the real CLI (recipe and sanitising in `src/providers/claude_code/CLAUDE.md`).

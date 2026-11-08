# Claude Code provider

## Purpose
Runs agents with the Claude Code CLI (`claude`) installed and logged in on the user's machine, in headless mode (ADR-0003, ADR-0027). This folder is the only code that knows Claude Code: flags, stream-json output, error wording, install locations.

## Command lines
| Use | Command |
|---|---|
| Version | `claude --version` → `2.1.317 (Claude Code)` (first numeric word kept) |
| Login | `claude auth status --json` → `{"loggedIn":…,"authMethod":…,"apiProvider":…,"subscriptionType"?…}`; exits 1 when logged out but still prints the JSON. Only `loggedIn`, `subscriptionType` and `authMethod` are read |
| Run | `claude -p --output-format stream-json --verbose --tools "" --permission-prompts none --setting-sources "" --restricted --disable-slash-commands --strict-mcp-config --append-system-prompt-file <workdir>/.simpleeffect-instructions.md [--mcp-config <json> --allowedTools mcp__simpleeffect__*] [--model m] [--effort e] (--session-id <uuid> \| --resume <uuid>)`, cwd = the task's working folder, prompt on stdin |

## Empirical validation (Claude Code 2.1.317, macOS, 2026-11-07)
Recorded with the CLI bundled with the Claude desktop app (the same build as the public 2.1.317), in a scratch folder.

**Flags**
- Every flag above is accepted together (unknown flags fail immediately with `error: unknown option`). `--append-system-prompt-file` exists although `--help` only mentions it under `--bare`.
- `--session-id <uuid>` is honoured: the `init` line and the `result` line carry that id.
- The prompt is read from stdin when `-p` has no prompt argument.

**Isolation** (observed on the `init` line and with `--debug-file`, in a clean environment (`env -i HOME PATH USER TMPDIR LANG`), using a temporary `CLAUDE_CONFIG_DIR` holding a `settings.json` with a `UserPromptSubmit` hook and `"model":"sonnet"`, and a `CLAUDE.md`)
| Flags | User settings (model, hooks) | Our `--mcp-config` server | Skills / slash commands | Auto-memory |
|---|---|---|---|---|
| none | applied (model = sonnet, **hook ran**) | kept | 18 / 48 | on |
| `--safe-mode` | model still applied | **dropped** | 18 / 48 | off |
| `--setting-sources ""` (or `local`) | ignored (hook did not run) | kept | 18 / 48 | on |
| `--restricted` | ignored | kept | 18 / 48 | off |
| `--setting-sources "" --restricted --disable-slash-commands` | ignored | kept | 0 / 0 | off |

Kept: the last row (+ `--strict-mcp-config`, `--tools ""`). Excluded: `--safe-mode` (drops our MCP server), `--bare` (requires an API key: OAuth/keychain never read, so subscription logins fail).

**Logged out** (`not-logged-in.jsonl`): the CLI still prints `system/init`, then a synthetic `assistant` message (`model: "<synthetic>"`) and a `result` with `subtype: "success"`, `is_error: true`, `result: "Not logged in · Please run /login"`, `terminal_reason: "api_error"`; exit code 1. Classified `notAuthenticated` (the message contains `/login`).

**To validate logged in** (TODO before relying on it — see `prompts/PROGRESS.md`): success and long-text fixtures, a resumed session (`--resume` from the same working folder after it was deleted and recreated), a lost session's error, and whether the global `~/.claude/CLAUDE.md` or an ancestor `CLAUDE.md` of the working folder reaches the agent with the kept flags.

**Environment.** A Claude Code session sets many variables for its children (`CLAUDECODE`, `CLAUDE_CODE_SIMPLE`, `CLAUDE_CODE_SESSION_ID`, `CLAUDE_CODE_ENTRYPOINT`, `CLAUDE_AGENT_SDK_*`, a local `ANTHROPIC_BASE_URL`…). SimpleEffect started from inside one (`pnpm dev` from Claude Code's terminal) would pass them on and make our CLI a nested child of that session. `environment.rs` removes them when `CLAUDECODE` is present, keeping user configuration (`CLAUDE_CODE_OAUTH_TOKEN`, `CLAUDE_CODE_USE_BEDROCK`/`VERTEX`/`FOUNDRY`, `CLAUDE_CODE_MAX_OUTPUT_TOKENS`, `ANTHROPIC_API_KEY`); started normally, the environment passes untouched. Record fixtures and run experiments with `env -i` for the same reason.

## stream-json → AgentEvent
| Line | Events |
|---|---|
| `{"type":"system","subtype":"init","session_id","model",…}` | `SessionStarted` (once per session id) |
| `{"type":"assistant","message":{"content":[…]}}` | `text` blocks → `AssistantText` (blank ones skipped); `tool_use` → `ToolCallStarted { call_id: id, tool: name, input_summary }`; `thinking` and others ignored |
| `{"type":"user","message":{"content":[{"type":"tool_result",…}]}}` | `ToolCallFinished { call_id: tool_use_id, tool (remembered), ok: !is_error, output_summary }` |
| `{"type":"result",…}` | `Usage { input_tokens (+ cache creation + cache read), output_tokens, cost_usd: total_cost_usd, duration_ms }`, then `Completed { summary: result }` when `subtype == "success"` and not `is_error`, else `Failed` classified from `result` / `errors` / `subtype` |
| anything else (`stream_event`, `rate_limit_event`, other `system` subtypes…) | ignored (debug log) |

Without a `result` line: stderr decides (`No conversation found` → lost session, login phrases → `notAuthenticated`, else `crashed` with the exit status); unreadable lines or a clean exit → `protocolError`.

## Recording fixtures
1. In a scratch folder named like a task id (`…/rec/0195f3a2-7c41-7d2e-9a10-3b4c5d6e7f80`), write `.simpleeffect-instructions.md`, then run the **Run** command above with `--session-id 0195f3a2-7c41-7d2e-9a10-3b4c5d6e7f81`, the prompt on stdin, stdout to `<name>.raw.jsonl`.
2. Sanitise: replace the scratch path with `/tmp/agent-workdirs/0195f3a2-7c41-7d2e-9a10-3b4c5d6e7f80`, `/Users/<name>` with `/Users/user`, `-Users-<name>-` with `-Users-user-`, null `messaging_socket_path`; compact each line. Never commit e-mails, organisation names or tokens.
3. Save as `crates/se-agents/tests/fixtures/claude_code/<name>.jsonl` and replay it in `tests/claude_code_cli.rs`.

## Rules
- Prompts and instructions never travel as arguments (Windows `.cmd` shims cannot carry line breaks).
- Keep the isolation flags together; re-run the isolation table when the CLI changes major behaviour, and update this file and ADR-0027.
- New failure phrases go in `failure.rs` with a test quoting the real message.

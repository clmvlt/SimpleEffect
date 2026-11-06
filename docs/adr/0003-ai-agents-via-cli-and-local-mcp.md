# 0003 — AI agents via local CLIs and an embedded MCP server

- Status: Accepted
- Date: 2026-11-06

## Context
The editor is AI-first. It must use the AI CLI already installed and logged in on the user's machine (Claude Code first), with no login inside the app, and must later support other CLIs. Several AI tasks must run in parallel with the user's own edits without corrupting the project. Everything the AI produces must remain editable.

## Decision
- se-core embeds an **MCP server** (`rmcp`, streamable HTTP at `/mcp`). AI agents act **only** through its tools; they never read or write project files.
- Each task gets a **per-task bearer token** bound to its scope (sequence, time range, tracks). Write tools reject out-of-scope edits with actionable errors.
- Tool calls become domain `Operation`s applied by the single-writer `ProjectActor` (ADR-0005), attributed to the task.
- AI CLIs are abstracted behind the `AgentProvider` trait (`detect`, `auth_status`, `capabilities`, `start` → event stream + cancel). The first implementation is **Claude Code** in headless mode:
  `claude -p … --output-format stream-json --verbose --mcp-config <inline json> --strict-mcp-config --tools "" --allowedTools "mcp__simpleeffect__*" --permission-prompts none --append-system-prompt … [--model] [--effort] (--session-id | --resume)`.
- Session state is checked with `claude auth status --json` (`loggedIn`). When logged out, AI features are disabled and the UI explains how to log in from a terminal.
- Agent instructions are provider-agnostic markdown files embedded in se-app.

## Consequences
- Adding another AI CLI means writing one adapter; tools and instructions are reused because MCP is supported by the major CLIs.
- Agents can see the project through read tools (timeline, transcript, audio analysis, extracted frames as images).
- Every AI change is validated, journaled and revertable as a unit.
- We depend on the CLI's flags and stream-json format: the adapter is isolated, version-checked and covered by replay tests with recorded fixtures.
- The app never touches AI credentials. Personal use only for now; distributing the app to other users would require an API-key mode and checking Anthropic's terms.

## Alternatives considered
- **Agent SDK / direct API calls**: require API keys or credential handling; contradicts "use the installed, logged-in CLI". Rejected.
- **Letting the agent edit `project.json` with built-in file tools**: races with the user and other tasks, no validation, no attribution. Rejected.
- **stdio MCP bridge binary per task**: works, but an HTTP endpoint inside se-core is simpler (no extra binary) and supported by the target CLIs. Rejected.

# se-agents

## Purpose
AI CLI providers and their stream parsing (`docs/adr/0003-ai-agents-via-cli-and-local-mcp.md`). Claude Code is the first provider. This crate launches the CLI, maps its streaming output to `AgentEvent`s and cancels runs; agents act on the project only through MCP tools (`se-mcp`).

Currently an empty skeleton, implemented by prompt 17.

## Public API
None yet. Planned: `AgentProvider` implementations under `src/providers/<provider>/`.

## Rules and invariants
- The **only** crate that knows which AI CLI is used (golden rule 3).
- Agents are launched with no built-in tools (`--tools ""` for Claude Code), in an empty working directory.
- Resolve executables with the `which` crate (PATHEXT-aware: `claude` is a `.cmd` shim on Windows) and support a user override path.
- Spawn without a console window; kill the whole process tree on cancel.
- Never log or persist CLI credentials; redact MCP task tokens.

## Allowed dependencies
`se-app`, `se-domain`, `tokio` (process), `which`, `serde_json`. Never another adapter crate. See `docs/architecture.md` §3 and §9.

## How to test
- `cargo test -p se-agents`
- Stream parsing is tested by replaying recorded CLI output fixtures.

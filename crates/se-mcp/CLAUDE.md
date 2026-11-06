# se-mcp

## Purpose
The MCP server exposing SimpleEffect tools to AI agents: the agents' only way to read and change a project. Tools translate MCP calls into `se-app` use cases; they never mutate documents themselves.

Currently an empty skeleton, implemented by prompts 18 (read tools) and 19 (write tools, scope, changesets).

## Public API
None yet. Planned: a tool router mounted by `se-core` at `/mcp`, one tool per file in `src/tools/<tool_name>.rs`.

## Rules and invariants
- One tool per file; write tools go through use cases and check the task scope.
- Tool inputs/outputs use seconds at the boundary; convert to `Flicks` with the dedicated helpers.
- Each tool has a unit test and a schema snapshot; the catalog in `docs/ai-tools.md` stays in sync.
- Per-task tokens authenticate agents; redact them in logs.

## Allowed dependencies
`se-app`, `se-domain`, `rmcp`, `schemars`. Never another adapter crate. See `docs/architecture.md` §3 and §9.4.

## How to test
- `cargo test -p se-mcp`
- Tool handlers are tested against `se-app` fakes; schemas with `insta` snapshots.

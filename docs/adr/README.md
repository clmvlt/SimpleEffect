# Architecture Decision Records

An ADR records one significant decision: its context, the choice, and its consequences.
ADRs are immutable once accepted: to change a decision, write a new ADR that supersedes the old one and update the old one's status.

| # | Title | Status |
|---|---|---|
| [0001](0001-electron-shell-rust-core-process.md) | Electron shell with a Rust core process | Accepted |
| [0002](0002-web-rendering-engine.md) | Web rendering engine shared by preview and export | Accepted |
| [0003](0003-ai-agents-via-cli-and-local-mcp.md) | AI agents via local CLIs and an embedded MCP server | Accepted |
| [0004](0004-time-in-flicks.md) | Time represented as integer flicks | Accepted |
| [0005](0005-single-writer-operations-changesets.md) | Single-writer actor, operations and live changesets | Accepted |
| [0006](0006-project-storage-layout.md) | Project storage layout | Accepted |
| [0007](0007-media-pipeline-sidecars-mezzanine.md) | Media pipeline with sidecar binaries, mezzanines and proxies | Accepted |
| [0008](0008-premiere-export.md) | Premiere Pro export strategy | Accepted |
| [0009](0009-design-system.md) | Design system `@se/ui` | Accepted |
| [0010](0010-typed-protocol-codegen.md) | Typed protocol with Rust as the source of truth | Accepted |
| [0011](0011-motion-templates.md) | AI-authored motion templates in a sandbox | Accepted |
| [0012](0012-rpc-protocol-conventions.md) | RPC protocol conventions: codegen binary, command/result pairing, event delivery | Accepted |
| [0013](0013-design-tokens-and-styling.md) | Design tokens and styling conventions of `@se/ui` | Accepted |
| [0014](0014-renderer-shell-i18n-and-editor-widgets.md) | Renderer shell, i18n and editor widget conventions | Accepted |
| [0015](0015-spatial-units.md) | Spatial units: normalised positions, design pixels on the shorter side | Accepted |

## Template

```markdown
# NNNN — Title

- Status: Proposed | Accepted | Superseded by NNNN
- Date: YYYY-MM-DD

## Context
What forces are at play? What problem are we solving?

## Decision
What we decided, stated plainly.

## Consequences
What becomes easier, what becomes harder, what we must now do.

## Alternatives considered
Each alternative and why it was rejected.
```

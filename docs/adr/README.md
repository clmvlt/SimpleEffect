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
| [0016](0016-operation-inverses-and-revert-semantics.md) | Operation inverses, revisions and changeset revert semantics | Accepted |
| [0017](0017-project-persistence-and-synchronisation.md) | Project persistence, locking and UI synchronisation | Accepted |
| [0018](0018-media-import-jobs-and-cache.md) | Media import, background jobs and the derived-file cache | Accepted |
| [0019](0019-mezzanines-proxies-and-media-analyses.md) | Mezzanines, proxies, media analyses and media settings | Accepted |
| [0020](0020-timeline-rendering-and-view-state.md) | Timeline rendering and view state | Accepted |
| [0021](0021-timeline-editing-commands-and-ripple-rules.md) | Timeline editing: intent commands, edit plans, placement and ripple rules | Accepted |
| [0022](0022-rendering-engine-evaluation-decoding-compositing.md) | Rendering engine: evaluation, frame-accurate decoding, compositing and golden frames | Accepted |
| [0023](0023-playback-audio-clock-and-av-sync.md) | Real-time playback: audio master clock, sequential decoding, chunked audio scheduling | Accepted |
| [0024](0024-transitions-fades-and-sound-levels.md) | Transitions on tracks that follow their items, fade shapes, gain envelopes and isolated dissolves | Accepted |
| [0025](0025-graphics-keyframes-presets-and-fonts.md) | Graphics, animatable styles, data-defined animation presets and the font catalogue | Accepted |
| [0026](0026-reframing-formats-and-sequence-versions.md) | Reframing, format changes, sequence versions and safe areas | Accepted |
| [0027](0027-ai-task-runtime.md) | AI task runtime: task service, runs and sessions, Claude Code invocation and isolation | Accepted |
| [0028](0028-mcp-server-and-read-tools.md) | MCP server, per-run tokens, read tools and frames rendered by the renderer | Accepted |
| [0029](0029-agent-edits-scope-and-changesets.md) | Agent edits: one edit service, scope checked before and after, changesets owned by the actor | Accepted |
| [0030](0030-ai-task-ux.md) | AI task UX: composer scopes, a renderer task mirror, patch actors and revert previews | Accepted |
| [0031](0031-local-transcription-whisper.md) | Local transcription with whisper.cpp: builds, consented downloads, word timings, storage | Accepted |
| [0032](0032-smart-cuts.md) | Smart cuts: one planning for text-based editing, fillers and silences, users and agents | Accepted |

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

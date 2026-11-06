# 0011 — AI-authored motion templates in a sandbox

- Status: Accepted
- Date: 2026-11-06

## Context
The user wants "complete motion design" from prompts with timecodes, and wants to re-edit results (e.g. change a text) without re-prompting. Declarative layers with keyframes cover common needs but not complex procedural animation.

## Decision
Two levels of motion design:
1. **Declarative items** (text, shapes, images, media) with keyframed properties, easing curves and a preset library. Fully editable in the inspector.
2. **Motion templates**: an AI agent writes a TypeScript module exporting `render(ctx, time, params)` plus a **parameter schema** (JSON Schema subset: string, number with range, color, boolean, enum, font, asset). The inspector generates controls from the schema, so the user edits texts, colours and timings without code.

Templates run in a dedicated Worker: no network (CSP), no DOM, no protocol client, a restricted 2D drawing API, a per-frame time budget. Render errors are reported back to the agent (through the MCP tool result) so it can fix its code. Templates are stored per project and can be saved to a global library together with brand kits.

## Consequences
- Agents can produce arbitrary 2D motion graphics while results stay parameter-editable and reusable.
- Executing generated code requires a strict sandbox and validation (compile + test render before acceptance).
- Templates render identically in preview and export because they run inside the engine.

## Alternatives considered
- **Declarative only**: safe but limits motion design. Rejected as the only level.
- **Lottie JSON authored by the AI**: verbose and error-prone for LLMs. Rejected.
- **HTML/CSS animations rendered to canvas**: no reliable frame-accurate DOM-to-canvas path. Rejected.

# se-mcp

## Purpose
The MCP server exposing SimpleEffect tools to AI agents: the agents' only way to read and change a project (ADR-0003, ADR-0028, ADR-0029). se-core mounts its streamable HTTP service at `/mcp`; every call comes from one running task (its bearer token resolves to an `AgentContext`). Tools translate MCP calls into `se-app` use cases; they never mutate documents themselves: write tools hand typed `AgentEdit`s to `se_app::AgentEditService`, which plans them, checks the task's scope and commits one AI transaction recorded in the task's changeset. Prompt 18: the server and the read tools; prompt 19: write tools, `apply_batch`, `get_task_changes`.

The contract with agents — conventions, every tool's input, output and errors — is `docs/ai-tools.md`. Keep it, the tools and the schema snapshots in step.

## Public API
| Item | Role |
|---|---|
| `mcp_service(McpDeps)` | The `rmcp` `StreamableHttpService` se-core nests at `/mcp` (sessions in memory, closed after 5 min idle) |
| `McpServer::new(McpDeps)` | The `ServerHandler`: `get_info` (name `simpleeffect`, short instructions), `list_tools`, `call_tool` (resolves the bearer token of the request's `http::request::Parts` → `ACCESS_REVOKED` when the run is over); `call(agent, name, arguments)` runs a tool without HTTP (tests); `tools()` |
| `McpDeps { projects, media, renders, tokens, edits }` | The se-app services tools read through, and `edits` (`AgentEditService`), the only way they write |
| `bearer_token(&HeaderMap)` | `Authorization: Bearer <token>` (also used by se-core's `/mcp` guard) |
| `tools::catalogue()` | Every `ToolEntry`, in the order agents see them |
| `registry::{McpTool, ToolEntry}` | The tool contract and its type-erased form |
| `output::{ToolOutput, ToolError, codes}` | Compact JSON + JPEG results; errors with stable codes (`INVALID_ARGUMENTS`, `INVALID_TIME`, `*_NOT_FOUND`, `NOT_ANALYSED`, `TOO_MANY_FRAMES`, `ACCESS_REVOKED`, `INTERNAL`; use-case errors keep their protocol code) |
| `time_io::{TimeInput, RangeInput, TimeOutput, RangeOutput}` | Times at the boundary: seconds or timecode in, `{timecode, seconds}` out |
| `context::{ToolContext, find_sequence, find_tracks, find_item, find_asset, default_range, whole_sequence}` | The call's services and caller; lookups with actionable errors listing what exists |
| `describe::{item_summary, asset_summary, track_summary, …}` | Compact descriptions shared by tools |
| `timing::{ClipWindow, merge, complement, spread}` | Source ↔ sequence time of clips, range sets, frames spread over a range |
| `writes::{WriteTool, WriteScene, Planned, NamedItem, run, apply, app_error, EditOutput, summaries}` | Write tools: `WriteTool::plan(scene, input)` → se-app `AgentEdit`s (+ which one creates the item, for batch names); `run::<T>` plans on the current document and applies; `app_error` maps refusals (`REVISION_CONFLICT`, `LOCKED_TRACK`, `*_NOT_FOUND`, `VALIDATION_ERROR`); `WriteScene::{time, duration, item ($name or id), item_kind, tracks, video_track}` |
| `write_inputs::{PositionInput, PointInput, NamedPosition, StrokeInput, AnimationInput, ValueInput, EasingInput, color, span, presets}` | Input pieces shared by write tools (named places, `#rrggbb`, `start` + `end`/`duration`, keyframe values and easings) |

## Layout
| File | Role |
|---|---|
| `server.rs` | `McpServer`, `mcp_service`, `bearer_token` |
| `registry.rs` | `McpTool` (input type → schema via rmcp's `schema_for_input`, `NAME`, `DESCRIPTION`, `READ_ONLY`, `call`) and `ToolEntry` (definition with annotations + boxed handler parsing the arguments) |
| `context.rs`, `output.rs`, `time_io.rs`, `describe.rs`, `timing.rs`, `writes.rs`, `write_inputs.rs` | Shared plumbing (see above) |
| `analyses.rs` | `AnalysisCache` (stored analyses loaded once per asset per call) and `clips_in` (media clips of a kind of track intersecting a range, with their `ClipWindow`) |
| `tools.rs`, `tools/<tool_name>.rs` | One tool per file. Reads: `get_project_overview`, `get_timeline`, `get_item`, `list_assets`, `get_frames`, `get_source_frames`, `get_audio_analysis`, `get_scene_changes`, `get_transcript` (stub), `list_animation_presets`, `get_brand_kit` (stub), `list_templates` (stub), `get_task_changes`. Writes: `add_text`, `add_shape`, `add_image`, `update_item`, `set_keyframes`, `apply_animation_preset`, `set_volume`, `move_item`, `trim_item`, `split_at`, `delete_items`, `ripple_delete_range`, `insert_gap`, `add_transition`, `add_marker`, `apply_batch` (dispatches to the others' `WriteTool::plan`, `BATCHABLE`) |

## Rules and invariants
- One tool per file; tools call se-app use cases (`ProjectService::project`, `MediaService::analysis`/`frame_jpeg`, `FrameRenderService::render`, `AgentEditService::apply`/`changes`…) and never mutate documents. Read tools are annotated `readOnlyHint: true`; write tools `READ_ONLY = false` (destructive).
- Write tools never check the scope themselves: `AgentEditService` checks the whole planned transaction (before and after). They only translate input (times at the task sequence's rate, names, `$name` references, styles) into `AgentEdit`s, purely, and can therefore run alone or inside `apply_batch` unchanged. Keyframe times are relative to the item's start; every other time is sequence time.
- Names of tools, arguments and output fields are `snake_case`; inputs refuse unknown fields (`#[serde(deny_unknown_fields)]`), so typos come back as `INVALID_ARGUMENTS` naming the field. Doc comments of input fields become the schema's descriptions: write them for the agent (default, unit, limit).
- Times cross the boundary as seconds or timecodes (`TimeInput`, `RangeInput`) and leave as `TimeOutput`; flicks never appear in tool I/O, except inside `get_item`'s `properties`, whose flicks fields are rewritten (`readable_time`).
- Outputs stay short: compact JSON, pagination (`offset`/`limit` → `next_offset`) or caps with totals, excerpts of long texts. Every output carries the ids and revisions a later write needs.
- Errors are `ToolError` results (`isError`), never panics; messages say what is wrong and what to do (which tool lists the ids, what exists, the valid range). Only an unknown tool name is a protocol error.
- Default sequence = the task's; default range = the task's range on that sequence, else the whole sequence (`default_range`).
- Tokens are never logged (`server.rs` logs tool, task id, duration and error code).

## Recipe: add a tool
1. Create `src/tools/<tool_name>.rs`: a unit struct, an `Input` (`Deserialize` + `JsonSchema`, `deny_unknown_fields`, doc comments on fields), serialisable output structs, and `impl McpTool` (`NAME`, `DESCRIPTION` saying what it returns and when to use it, `READ_ONLY`, `call`). Read through `context.project()` and se-app use cases; return `ToolOutput::json(&…)` (+ `push_text`/`push_jpeg`) or a `ToolError`.
2. Declare the module in `src/tools.rs` and add `ToolEntry::of::<…>()` to `catalogue()` (bump the counts in its test).
3. Test it: pure parts in-file; behaviour in `tests/tools.rs` over the example project (`tests/support/mod.rs`: `Bed::open()`, `json`, `error_code`, `set_audio_analysis`, `set_scenes`).
4. `INSTA_UPDATE=always cargo test -p se-mcp --test schemas`, review the new `tests/snapshots/schemas__<tool>.snap`.
5. Add its section to `docs/ai-tools.md` (input table, output example, errors) and its row to the catalogue table (`the_catalogue_is_documented` fails otherwise); mention it in `crates/se-app/assets/agent-instructions/` when agents need guidance.
6. A write tool: `READ_ONLY = false`, `call` = `writes::run::<Self>(context, input)`, `impl WriteTool` (its input as `AgentEdit`s; `Planned.creates` names the edit creating its item), add it to `apply_batch`'s `BATCHABLE` and `plan_action` (and to `WRITE_TOOLS` in the catalogue test). A new kind of edit needs an `AgentEdit` variant and its planner in se-app's `agent_edits/`.

## Allowed dependencies
`se-app`, `se-domain`, `se-protocol` (DTOs read through se-app: scopes, presets, error codes), `rmcp` (server + streamable HTTP server), `schemars`, `serde`/`serde_json`, `http` (request parts), `base64` (images), `tokio`, `tracing`. Never another adapter crate. See `docs/architecture.md` §3 and §9.4.

## How to test
- `cargo test -p se-mcp`
- Unit tests in-file: time parsing and output, ranges, lookups, summaries, clip windows and range sets, readable item properties, loudness downsampling, bearer tokens, tool definitions (object schemas, unique names, read-only).
- `tests/tools.rs`: every read tool against the example project opened over se-app's `MediaTestbed` (analyses seeded in the fake folder, frames delivered to the fake renderer link).
- `tests/write_tools.rs`: write tools on the example project (creations with animations, refusal codes, timing tools, keyframes and presets, batches with references and atomicity, `get_task_changes` of a real task via `Bed::run_task`) and the replay of `tests/fixtures/subscribe-pop.json` (`add_text` + `apply_animation_preset`) and `subscribe-pop-recorded.json` (the calls of a real Claude Code run; `{created.N}` placeholders): the item is snapshotted (`snapshots/write_tools__subscribe_pop*_item.snap`), then `RevertTask` restores the initial project. Record new fixtures from the task log (`getTaskEvents`: `toolCallStarted.inputSummary` holds the arguments).
- `tests/schemas.rs`: insta snapshots of every tool definition (`tests/snapshots/`) and a check that `docs/ai-tools.md` has a section per tool.
- The HTTP side (token guard, real rmcp client, images) is tested in se-core (`src/mcp_routes.rs`).

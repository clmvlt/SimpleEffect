# se-protocol

## Purpose
Data transfer objects exchanged between `se-core` and its clients: the startup handshake line and the JSON messages of the `/rpc` WebSocket. DTOs only: no logic beyond serde (de)serialisation and trivial constructors. Business rules live in `se-domain` and `se-app`. Every type derives `ts_rs::TS` and is exported to `@se/protocol` by `pnpm codegen` (ADR-0010, ADR-0012).

## Public API
| Module | Types |
|---|---|
| `handshake.rs` | `CoreHandshake { port, token, version }`: the single stdout line of `se-core`. `Debug` redacts the token |
| `envelope.rs` | `ClientMessage::Request(RequestEnvelope { id, command })`; `ServerMessage::Response(ResponseEnvelope { id, outcome })` / `ServerMessage::Event(EventEnvelope { event })`; `ResponseOutcome` (`result` \| `error`); `RequestId` (`u64`, exported as `number`) |
| `command.rs` | `Command`: `ping`, `getCoreInfo`, project commands (`getNewProjectDefaults`, `createProject(CreateProjectRequest)`, `openProject`, `closeProject`, `listRecentProjects`, `renameProject`, `duplicateProject`, `trashProject`, `getProjectSnapshot`, `createSequence`, `undo`, `redo`), media commands (`importMedia(ImportMediaRequest)`, `relinkAsset(RelinkAssetRequest)`, `removeAsset`, `listJobs`, `cancelJob`, `reportDecodability(DecodabilityReport)`, `reportDecoderCapabilities(DecoderCapabilities)`, `generateDerivedMedia(GenerateDerivedRequest)`, `getMediaSettings`, `updateMediaSettings(MediaSettings)`, `getMediaToolsStatus`, `getProjectCache`, `clearProjectCache`) |
| `command_result.rs` | `CommandResult` (same kinds; sessions, recent projects, defaults…), `CoreInfo { version, os, arch, appDataDir, logsDir }` |
| `event.rs` | `CoreEvent`: `coreReady { version }`, `coreShuttingDown`, `resyncRequired { missedEvents }`, `projectOpened(ProjectSession)`, `projectPatched(ProjectPatch)`, `projectClosed { projectId }`, `historyChanged(ProjectHistory)`, `jobQueued { job }`, `jobProgress(JobProgress)`, `jobCompleted { job }`, `jobFailed { job }`, `jobCancelled { job }` |
| `media.rs` | `ImportMediaRequest`, `ImportReport { imported, duplicates: [DuplicateMedia], failures: [ImportFailure { path, code, message }] }`, `RelinkAssetRequest`, `RelinkOutcome` (`relinked` \| `contentDiffers`), `DecodabilityReport`, `ThumbnailSheet` (JSON of `/media/assets/{id}/thumbnails`, a codegen root) |
| `media_settings.rs` | `MediaSettings { mezzanineQuality, proxyResolution, autoProxies, toolsDir }` (+ `MEZZANINE_QUALITY_RANGE` 8–28, default 14), `ProxyResolution` (`540p`, `720p`), `DecoderCapabilities { hevcMain10, h264High, aac, flac }`, `MediaToolsStatus`, `DerivedKind` (`mezzanine`, `proxy`), `GenerateDerivedRequest`, `ProjectCacheInfo { path, sizeBytes }` |
| `jobs.rs` | `JobKind` (`thumbnails`, `waveformPeaks`, `mezzanine`, `proxy`, `audioAnalysis`, `sceneDetection`), `JobState` (`queued`, `running { progress }`, `completed`, `failed { message }`, `cancelled`), `JobSnapshot`, `JobProgress` |
| `project_entities.rs` | `ProjectSnapshot { revision, project: ProjectHeader, sequences, tracks, items, markers, assets }`, `SequenceEntity`, `TrackEntity`, `ItemEntity` / `MarkerEntity` (domain type `#[serde(flatten)]` + parent id), `ProjectPatch { projectId, revision, upserts: EntityUpserts, deletes: EntityDeletes }` |
| `project_session.rs` | `ProjectSession { path, snapshot, history }`, `HistoryState`, `ProjectHistory`, `RecentProject`, `NewProjectDefaults`, `FormatPresetDimensions`, `CreateProjectRequest` |
| `error.rs` | `ProtocolError { code, message, details? }` (+ `with_detail(key, value)`), `ErrorCode` (incl. project codes `PROJECT_NOT_FOUND`, `PROJECT_LOCKED`, `SCHEMA_TOO_NEW`, `PROJECT_CORRUPTED`, `PROJECT_ALREADY_EXISTS`, `PROJECT_NOT_OPEN`, `STORAGE_FAILED`, `EDIT_REJECTED`, `NOTHING_TO_UNDO`, `NOTHING_TO_REDO`) and media codes (`ASSET_NOT_FOUND`, `MEDIA_NOT_FOUND`, `MEDIA_UNSUPPORTED`, `MEDIA_KIND_MISMATCH`, `MEDIA_TOOLS_MISSING`, `JOB_NOT_FOUND`, `DERIVED_NOT_APPLICABLE`, `MEDIA_TIME_OUT_OF_RANGE`) |

Wire format (`docs/architecture.md` §6):
```json
{"type":"request","id":1,"command":{"kind":"getCoreInfo"}}
{"type":"response","id":1,"result":{"kind":"getCoreInfo","version":"0.1.0","os":"windows",…}}
{"type":"response","id":1,"error":{"code":"UNKNOWN_COMMAND","message":"…","details":{"kind":"explode"}}}
{"type":"event","event":{"kind":"resyncRequired","missedEvents":3}}
```

## Rules and invariants
- Enums are internally tagged: `type` for messages, `kind` (camelCase) for commands, results and events. Struct-variant fields are camelCase (`rename_all_fields`).
- A newtype variant flattens its payload next to `kind`: a payload with its own `kind` field (`JobSnapshot`) must be wrapped in a struct variant (`JobCompleted { job }`), or its field silently overwrites the tag (a test in `event.rs` guards the job events).
- **Every `Command` variant has a `CommandResult` variant with the same name** (same wire `kind`): the TypeScript client derives result types from it. Put a non-trivial payload in its own struct (`GetCoreInfo(CoreInfo)`).
- Error codes are stable (`SCREAMING_SNAKE_CASE`): never rename or reuse one. Messages are actionable (read by users and AI agents). `TIMEOUT`, `CONNECTION_LOST`, `CLIENT_CLOSED` are raised by clients, never sent by `se-core`.
- Integers that can exceed 2^53 must not appear in DTOs: `u64`/`i64` are exported as `number` (revisions, millisecond dates).
- Entities sent to clients are whole and normalised (ADR-0017): parents list children ids, children carry their parent id. A new entity kind goes in `ProjectSnapshot`, `EntityUpserts`/`EntityDeletes`, se-app's patch builder and the renderer's mirror.
- Paths travel as absolute strings. Commands/results/events that carry no `Eq` type (floats in items) derive `PartialEq` only.
- Adding an error code: add it here, then to `ERROR_CODES` and `fr.json` (`errors.<CODE>`) in the renderer (a type test fails otherwise).
- Optional fields that may be omitted on the wire use `#[serde(default, skip_serializing_if = "Option::is_none")]` + `#[ts(optional)]`.
- Never log a `CoreHandshake` token or any session token.
- After **any** change here: `pnpm codegen`, then commit `packages/protocol/src/generated/` (`pnpm check` fails otherwise).

## Recipe: add a command
1. `command.rs`: add the variant to `Command` (doc comment naming its result).
2. `command_result.rs`: add the variant **with the same name** to `CommandResult` (unit, or wrapping a new payload struct deriving `TS`). Add a JSON-shape test.
3. `crates/se-core/src/rpc/handlers/<group>.rs`: write the handler (returns `CommandResult`, or `Result<CommandResult, ProtocolError>` if it can fail; new error codes go in `ErrorCode`).
4. `crates/se-core/src/rpc/router.rs`: add the match arm in `dispatch` (the compiler points at it); extend `every_result_has_the_kind_of_its_command`.
5. `pnpm codegen`; the renderer can now call `client.send({ kind: '<newCommand>', … })` with a typed result.
6. Document it in `docs/architecture.md` §6 if it introduces a new concept.

Adding an event: add the variant to `CoreEvent`, publish it with `EventBus::publish` in se-core, run `pnpm codegen`; clients subscribe with `client.on('<kind>', handler)`.

## Allowed dependencies
`se-domain`, `serde`, `serde_json` (for `details`), `ts-rs`. Dev: `se-domain` with `test-support`. Nothing else; see `docs/architecture.md` §3.

## How to test
- `cargo test -p se-protocol`
- Unit tests in-file assert the exact JSON shape of each message and round-trip every envelope.

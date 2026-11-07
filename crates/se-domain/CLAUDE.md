# se-domain

## Purpose
The pure document model of SimpleEffect: time in flicks, identifiers, projects, sequences, tracks, items, keyframes, validation, and every change of a document: operations with exact inverses, transactions, the user's undo/redo history and AI-task changesets with conflict-aware reverts (ADR-0005, ADR-0016). It is the single source of truth for document types (exported to TypeScript through ts-rs). It does **not** persist, schedule, render or talk to anything: the `ProjectActor` of se-app (prompt 07) owns the project and calls `commit`.

## Public API
Everything is re-exported at the crate root (`se_domain::Flicks`, `se_domain::Project`…).

| Module | Contents |
|---|---|
| `time` | `Flicks` (`PER_SECOND`, `MAX_SAFE` = 2^53 − 1, checked arithmetic, seconds/frames conversions), `FrameRate` (reduced, 1–1000 fps, exact frame duration, `STANDARD` constants, `nominal_fps`), `RoundingMode`, `TimeRange` (half-open), `parse_timecode` / `format_timecode`, `TimeError`, `TimecodeError` |
| `ids` | `ProjectId`, `SequenceId`, `TrackId`, `ItemId`, `AssetId`, `TaskId`, `ChangesetId`, `TemplateId`, `LinkGroupId`, `MarkerId`, `TransactionId`, `JobId` (UUIDv7 newtypes: `parse`, `from_uuid`, `as_uuid`, `from_test_index`), `IdError`, `Uuid` |
| `spatial` | `Vec2`, `DesignPixels`, `DESIGN_REFERENCE_SIDE`, `design_scale` (ADR-0015) |
| `color` | `Color` (`#rrggbb[aa]`), `ColorParseError` |
| `document` | `Project`, `ProjectSettings`, `Asset`, `ContentFingerprint` (validated sample hash, `same_content`, `cache_key`), `Decodability` / `StreamDecodability` (per stream, `unchecked(info)`), `MediaInfo`, `DerivedMedia`, `Sequence`, `Format`, `FormatPreset`, `Track`, `Item`, `Origin`, `ItemKind` and its payloads (`MediaClip`, `TextItem`, `ShapeItem`, `ImageItem`, `TemplateItem`, `CaptionItem`), `Transform2D`, `Animatable`, `Keyframe`, `Easing`, `Marker`, `MotionTemplate`, `BrandKit` |
| `validation` | `validate_project(&Project) -> Vec<Violation>`, `Violation { code, path, message }`, `ViolationCode` (17 codes, table in `docs/architecture.md` §5.3) |
| `operation` | `Operation` (26 payload structs: `AddItem`, `MoveItem`, `TrimItem`, `RippleDeleteRange`…, list in `docs/architecture.md` §5.4), `apply(&Project, &Operation) -> Result<Applied { project, inverse, touched }, DomainError>`, `TouchedEntities`, patches (`ItemPatch` and one patch per kind, `TrackPatch`, `SequencePatch`, `MarkerPatch`, `AssetPatch`, `Assign`), `PropertyAnimation`, `AnimatableProperty`, `ItemEdge` |
| `transaction` | `Transaction { id, actor, label, ops }`, `Actor` (`user`, `aiTask`, `system`: journaled, never undone), `commit(&Project, &Transaction) -> Result<Committed { project, inverse, touched }, DomainError>` (all or nothing, bumps revisions) |
| `history` | `History` (user undo/redo: `record`, `undo`, `redo`, labels), `HistoryStep` |
| `changeset` | `Changeset`, `ChangesetEntry`, `plan_revert(&Project, &Changeset) -> RevertPlan { ops, conflicts }`, `Conflict` |
| `error` | `DomainError` (actionable messages, times as `HH:MM:SS.mmm`), `DomainErrorCode` (`code()`) |
| `test_support` | feature `test-support`: `example_project()` (valid, every item kind), `example_ids`, `millis`, `range_ms`, `item`, `track` |

## Layout
- One concept per file: `time/*.rs`, `document/*.rs`, `validation/*_rules.rs` (one family of rules each, with one test per code in the same file), `operation/*.rs` (one operation or small group each, with its unit tests).
- `validation/report.rs` holds the shared plumbing (`Path`, `Report`, `ProjectIndex`, `seconds()` for messages).
- `operation/` plumbing: `effect.rs` (`Edit` trait implemented by every payload, `Effect` = inverse + touched, `run_steps` for composite edits), `lookup.rs` (find entities by id), `checks.rs` (locked track, revision, overlap, unused id, names, indices, `swap_field` for inverse patches), `placement.rs` (`place_item`, `insert_new_item`, `remove_item`: the only code that moves items between tracks), `reshape.rs` (retiming keyframes, fades and source of a trimmed or cut item), `testing.rs` (unit-test helpers, `cfg(test)`).

## Rules and invariants
- Pure: no IO, no async, no clock, no randomness, no process spawning, no logging side effects.
- Time is always `Flicks`; never floats for positions or durations. Use the conversion helpers of `time`.
- **Single-value invariants live in the value types** (private fields + validating constructor + validating `Deserialize`): `Flicks` range, `TimeRange` duration, `FrameRate`, ids, `Color`. **Cross-entity invariants live in `validation`** and are reported, never panicked on. A new invariant goes in one of the two, with a test per violation code.
- Serde conventions: `rename_all = "camelCase"`; enums with data `tag = "kind"` (+ `rename_all_fields = "camelCase"` when variant fields have several words); unit enums as camelCase strings; `None` as `null`. Validated types implement `Deserialize` by hand (ts-rs ignores and warns about `#[serde(try_from)]`).
- Every serialized type derives `ts_rs::TS`. Types not reachable from `Project` or `Violation` need a root in `crates/se-codegen/src/bindings.rs`. Run `pnpm codegen` after any change; never hand-edit the generated TypeScript.
  Operations, transactions, changesets and conflicts derive `TS` but have no root yet: the renderer receives whole entities, never operations (ADR-0005). Add a root when a protocol command or event carries one (e.g. revert conflicts, prompt 20).
- The insta snapshot `src/document/snapshots/*example_project_json_format.snap` fixes the `project.json` format. If it changes, the format changed: think about schema migration (prompt 07) and the renderer, then accept it (`INSTA_UPDATE=always cargo test -p se-domain`).
- Messages of errors and violations are read by users and AI agents: say what is wrong and how to fix it, quote values, write times with `seconds()` (violations) or `error::clock()` (`HH:MM:SS.mmm`, domain errors).
- `apply` validates the result (`validate_project`, so it needs a valid input) and never changes revisions; `commit` bumps the project revision and the touched items' revisions once per transaction. Revisions only grow.
- Every operation records its exact inverse: item edits record `RestoreItem` of the previous state (via `place_item`), additions `RemoveItem`, removals `AddItem`/`Add*` at the same index. Composite edits build primitive steps with `run_steps`, ordered so that every intermediate state is valid (the inverse replays them backwards, validated one by one).
- Ids of new entities are always inputs (`newItemId`, `splitIds`, payload ids): never generate ids here.
- Track items stay sorted by start (`placement::insert_sorted`); never push onto `track.items` directly.

### Recipe: add an operation
1. Create `src/operation/<name>.rs` with a payload struct (`Serialize`, `Deserialize`, `TS`, `rename_all = "camelCase"`, doc comment stating what it does and its inverse) and `impl Edit`: check preconditions with `lookup`/`checks` (unknown id, locked track, `expected_revision`…), mutate through `placement` for items, call `effect.undo_with(…)` with the exact inverse and fill `effect.touched`.
2. Add the variant (with a doc line) to `Operation` and the name to `operation_payloads!` in `src/operation.rs`; re-export the payload in `operation.rs` and `lib.rs`.
3. New refusal → a `DomainError` variant with an actionable message and its `DomainErrorCode`.
4. Unit tests in the file with `operation::testing::apply_ok` (asserts a valid result and an exact inverse) and `apply_err`; add a case to `tests/operations/generators.rs` (bump `KINDS`) so the property tests cover it.
5. Update `docs/architecture.md` §5.4; the journal snapshot (`src/snapshots/*transaction*`) changes only if an existing operation's format changes (stored journals then need a migration, prompt 07).

## Allowed dependencies
No workspace crate (innermost layer). Third-party: `serde`, `thiserror`, `uuid`, `ts-rs` (derive only). Dev: `proptest`, `insta`, `serde_json`. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-domain` (unit tests see `test_support` through `cfg(test)`; no feature needed).
- Unit tests in-file under `#[cfg(test)]`; validation tests mutate `test_support::example_project()` and expect exactly one code.
- `tests/test_vectors.rs` runs `docs/test-vectors/flicks.json` and `timecode.json` (also used by the TypeScript time helpers of prompt 10: change both sides together).
- `tests/time_properties.rs`: proptest (frame round trips at every standard rate, `TimeRange` algebra, timecode round trips).
- `tests/operations/` (1000 cases each): random operations from `generators.rs` keep projects valid and invert exactly; undo/redo walk through the same states with growing revisions; an undisturbed changeset revert restores the project. `every_operation_kind_succeeds_sometimes` guards against a vacuous generator.
- Integration tests reach `test_support` through the self dev-dependency with the `test-support` feature (`Cargo.toml`).
- Other crates' tests enable the `test-support` feature in their dev-dependencies to reuse the example project and deterministic ids.

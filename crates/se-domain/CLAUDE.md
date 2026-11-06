# se-domain

## Purpose
The pure document model of SimpleEffect: time in flicks, identifiers, projects, sequences, tracks, items, keyframes and validation (operations, transactions and changesets arrive with prompt 06). It is the single source of truth for document types (exported to TypeScript through ts-rs). It does **not** persist, schedule, render or talk to anything.

## Public API
Everything is re-exported at the crate root (`se_domain::Flicks`, `se_domain::Project`…).

| Module | Contents |
|---|---|
| `time` | `Flicks` (`PER_SECOND`, `MAX_SAFE` = 2^53 − 1, checked arithmetic, seconds/frames conversions), `FrameRate` (reduced, 1–1000 fps, exact frame duration, `STANDARD` constants, `nominal_fps`), `RoundingMode`, `TimeRange` (half-open), `parse_timecode` / `format_timecode`, `TimeError`, `TimecodeError` |
| `ids` | `ProjectId`, `SequenceId`, `TrackId`, `ItemId`, `AssetId`, `TaskId`, `ChangesetId`, `TemplateId`, `LinkGroupId`, `MarkerId` (UUIDv7 newtypes: `parse`, `from_uuid`, `as_uuid`, `from_test_index`), `IdError`, `Uuid` |
| `spatial` | `Vec2`, `DesignPixels`, `DESIGN_REFERENCE_SIDE`, `design_scale` (ADR-0015) |
| `color` | `Color` (`#rrggbb[aa]`), `ColorParseError` |
| `document` | `Project`, `ProjectSettings`, `Asset`, `MediaInfo`, `DerivedMedia`, `Sequence`, `Format`, `FormatPreset`, `Track`, `Item`, `Origin`, `ItemKind` and its payloads (`MediaClip`, `TextItem`, `ShapeItem`, `ImageItem`, `TemplateItem`, `CaptionItem`), `Transform2D`, `Animatable`, `Keyframe`, `Easing`, `Marker`, `MotionTemplate`, `BrandKit` |
| `validation` | `validate_project(&Project) -> Vec<Violation>`, `Violation { code, path, message }`, `ViolationCode` (15 codes, table in `docs/architecture.md` §5.3) |
| `test_support` | feature `test-support`: `example_project()` (valid, every item kind), `example_ids`, `millis`, `range_ms`, `item`, `track` |

## Layout
- One concept per file: `time/*.rs`, `document/*.rs`, `validation/*_rules.rs` (one family of rules each, with one test per code in the same file).
- `validation/report.rs` holds the shared plumbing (`Path`, `Report`, `ProjectIndex`, `seconds()` for messages).

## Rules and invariants
- Pure: no IO, no async, no clock, no randomness, no process spawning, no logging side effects.
- Time is always `Flicks`; never floats for positions or durations. Use the conversion helpers of `time`.
- **Single-value invariants live in the value types** (private fields + validating constructor + validating `Deserialize`): `Flicks` range, `TimeRange` duration, `FrameRate`, ids, `Color`. **Cross-entity invariants live in `validation`** and are reported, never panicked on. A new invariant goes in one of the two, with a test per violation code.
- Serde conventions: `rename_all = "camelCase"`; enums with data `tag = "kind"` (+ `rename_all_fields = "camelCase"` when variant fields have several words); unit enums as camelCase strings; `None` as `null`. Validated types implement `Deserialize` by hand (ts-rs ignores and warns about `#[serde(try_from)]`).
- Every serialized type derives `ts_rs::TS`. Types not reachable from `Project` or `Violation` need a root in `crates/se-codegen/src/bindings.rs`. Run `pnpm codegen` after any change; never hand-edit the generated TypeScript.
- The insta snapshot `src/document/snapshots/*example_project_json_format.snap` fixes the `project.json` format. If it changes, the format changed: think about schema migration (prompt 07) and the renderer, then accept it (`INSTA_UPDATE=always cargo test -p se-domain`).
- Messages of errors and violations are read by users and AI agents: say what is wrong and how to fix it, quote values, write times with `seconds()`.
- Every `Operation` (prompt 06) will have an inverse; property tests check `apply ∘ inverse = identity` and invariants.

## Allowed dependencies
No workspace crate (innermost layer). Third-party: `serde`, `thiserror`, `uuid`, `ts-rs` (derive only). Dev: `proptest`, `insta`, `serde_json`. See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-domain` (unit tests see `test_support` through `cfg(test)`; no feature needed).
- Unit tests in-file under `#[cfg(test)]`; validation tests mutate `test_support::example_project()` and expect exactly one code.
- `tests/test_vectors.rs` runs `docs/test-vectors/flicks.json` and `timecode.json` (also used by the TypeScript time helpers of prompt 10: change both sides together).
- `tests/time_properties.rs`: proptest (frame round trips at every standard rate, `TimeRange` algebra, timecode round trips).
- Other crates' tests enable the `test-support` feature in their dev-dependencies to reuse the example project and deterministic ids.

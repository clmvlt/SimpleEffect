# se-domain

## Purpose
The pure document model of SimpleEffect: time in flicks, identifiers, projects, sequences, tracks, items, keyframes, operations, transactions and validation. It is the single source of truth for document types (exported to TypeScript through ts-rs). It does **not** persist, schedule, render or talk to anything.

Currently an empty skeleton: the model arrives with prompts 05 (time and document model) and 06 (operations, history, changesets).

## Public API
None yet. Planned (see `docs/architecture.md` §5): `Flicks` and its conversion helpers, id newtypes (`ItemId`, …), `Project` and its children, `Operation`, `Transaction`, validation errors.

## Rules and invariants
- Pure: no IO, no async, no clock, no randomness, no process spawning, no logging side effects.
- Time is always `Flicks` (i64, 705,600,000 per second); never floats for positions or durations.
- Every `Operation` has an inverse; property tests check `apply ∘ inverse = identity` and invariants.
- Types shared with the renderer derive `ts_rs::TS`; never hand-edit the generated TypeScript.

## Allowed dependencies
No workspace crate (innermost layer). Third-party: `serde`, `thiserror`, `uuid`, `ts-rs` (derive only). See `docs/architecture.md` §3.

## How to test
- `cargo test -p se-domain`
- Unit tests in-file under `#[cfg(test)]`, `proptest` for invariants, `insta` for snapshots.

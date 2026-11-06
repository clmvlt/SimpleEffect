# 0005 — Single-writer actor, operations and live changesets

- Status: Accepted
- Date: 2026-11-06

## Context
The user edits while several AI tasks edit the same project in parallel. Each AI task's result must be revertable as a whole, even after other edits happened. The user chose **direct application** of AI changes (visible live while the agent works) rather than a review/draft mode.

## Decision
- Every mutation is an intent-level **`Operation`** with a pure `apply` that returns the inverse operations. Operations are grouped in atomic **transactions** tagged with an **actor** (`User` or `AiTask(id)`).
- One **`ProjectActor`** (tokio task) per open project is the only writer. It validates, applies, journals, bumps the project revision and emits entity-level patches.
- **User undo/redo** covers the user's own transactions only.
- All transactions of an AI task form a **changeset**, applied live. "Revert task" applies the changeset's inverse against the current state; items modified by another actor since then are reported as conflicts and skipped. A revert is a user transaction (undoable).
- Each item has a `revision`; agent writes can pass `expected_revision` and get `REVISION_CONFLICT` when stale.
- The UI receives whole-entity upserts/deletes, never operations, so no logic is duplicated in TypeScript.

## Consequences
- No data races or corruption; a total order of changes; complete history for crash recovery.
- Selective revert requires careful inverse computation and conflict reporting (property-tested).
- A draft/review mode can be added later by evaluating "document + pending changeset" without changing the storage model.

## Alternatives considered
- **CRDT (Loro, Automerge, Yjs)**: designed for multi-replica collaboration; overkill for one machine with one writer and harder to reason about. Kept as an option if multi-machine collaboration ever appears.
- **Locks per region**: blocks the user while an AI works. Rejected in favour of scope checks and optimistic concurrency.
- **Global interleaved undo stack**: undoing would revert AI work unpredictably. Rejected.

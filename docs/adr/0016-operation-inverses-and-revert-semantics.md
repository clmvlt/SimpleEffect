# 0016 — Operation inverses, revisions and changeset revert semantics

- Status: Accepted
- Date: 2026-11-07

## Context
ADR-0005 decided that every change is an invertible `Operation`, grouped in transactions, with user undo/redo and per-task changesets revertable after interleaved edits. Prompt 06 implemented it in `se-domain` and had to settle details that ADR-0005 leaves open: how inverses stay exact (rounding of speeds, keyframes dropped by a trim, fades shortened, items split by ripple edits), when revisions change, what a revert does when the document moved on, and what undo does with edits that changed nothing.

## Decision
- **`apply` never changes revisions; `commit` bumps them once per transaction.** The project revision grows by 1; every touched item that still exists grows by 1. An operation keeps the current revision of the items it modifies, so `apply(op)` then its inverse gives back exactly the original project, and revisions only ever grow (an item re-added by an undo comes back with its old revision, then bumped).
- **Item edits are inverted by snapshots.** Move, trim, update, keyframes and links are undone by `RestoreItem { track, item }`, which puts the recorded state back (keeping the current revision). Removal is undone by `AddItem` of the removed item, addition by `RemoveItem`. No inverse recomputes media offsets or keyframes, so rounding and dropped data cannot drift.
- **Composite edits are sequences of primitive steps.** `SplitItem`, `RippleDeleteRange` and `InsertGap` compute their result as `RestoreItem` / `AddItem` / `RemoveItem` steps ordered so that every intermediate state is valid, and their inverse is the steps' inverses in reverse order. Ids of new parts are given by the caller (`newItemId`, `splitIds`), keeping `apply` pure.
- **Edges keep their fades, cuts get none.** A trimmed edge keeps its fade (shortened to fit); the edges created by a cut start with no fade; the new right part of a split is in no link group.
- **Track items are sorted by start**, now a validation rule (`ITEMS_NOT_SORTED`), so an item's position in its track is deterministic and inverses are exact. Link groups stay within one sequence (`LINK_GROUP_ACROSS_SEQUENCES`).
- **Locked tracks**: no operation adds, changes or removes items of a locked track, or removes a locked track (or a sequence holding one); its header can still change. Sequence-wide edits with `tracks: null` skip locked tracks.
- **Revert = dry run against the current document.** `plan_revert` reports three kinds of `Conflict`: `itemChanged` (the item's revision differs from the one the task left: the task's edits of it are kept), `foreignItems` (an inverse would also change or remove items the task never touched, such as the user's items on a track the task created) and `cannotApply` (the inverse no longer applies, e.g. it would overlap a newer item). It applies the remaining inverses last transaction first, each on the result of the previous ones, so the returned operations are guaranteed to commit; the revert is then committed as a user transaction (undoable).
- **History stores inverse operations, not transactions.** Undo commits an entry's inverse ops as a new user transaction (journaled like any other) and pushes the inverse of *that* commit for redo. A failing undo or redo keeps its entry. Transactions whose inverse is empty (a ripple delete over empty time) are not recorded.

## Consequences
- Exactness is property-tested (`crates/se-domain/tests/operations/`): random operations keep projects valid and invert exactly; undo/redo walk the same states; an undisturbed changeset revert restores the project.
- Inverse transactions carry full item snapshots: journal entries of undos are larger than intent-level operations. Acceptable for SQLite (prompt 07); compaction comes with snapshots.
- Tracks, sequences, markers, assets and the project name have no revision: a revert restores the fields the task changed even if someone else changed the same field since (field-level patches keep other fields). Recorded in `docs/backlog.md`.
- A revert computes one `apply` (clone + validation) per inverse operation: linear in changeset size × project size, fine for the project sizes targeted.
- Linked items do not move together at the domain level: link-aware edits are use cases that emit one operation per linked item (prompt 11).

## Alternatives considered
- **Field-level inverses for every item edit** (inverse trim = trim back): inexact with speeds (rounding) and lossy (keyframes dropped, fades shortened). Rejected for snapshots.
- **Restoring whole tracks** as the inverse of ripple edits: simple but a revert would delete items added by others on those tracks. Rejected for per-item steps.
- **Revision bump inside `apply`**: breaks `apply ∘ inverse = identity` and bumps an item several times per transaction. Rejected.
- **Recording no-op transactions in the undo stack**: their undo is an empty transaction, which `commit` refuses. Rejected.

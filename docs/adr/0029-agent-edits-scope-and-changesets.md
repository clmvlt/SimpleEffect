# 0029 — Agent edits: one edit service, scope checked before and after, changesets owned by the actor

- Status: Accepted
- Date: 2026-11-08

## Context
ADR-0003 and ADR-0005 decided that AI agents change a project only through MCP tools, inside the
scope of their task, as transactions attributed to the task and revertable as a unit after
interleaved user edits; ADR-0016 fixed the domain side (changesets, `plan_revert`, conflicts) and
ADR-0028 built the MCP server with read tools. Prompt 19 gives agents write tools. Open questions:
where agent edits are planned and checked, what « in scope » means exactly (an item crossing the
range, a ripple edit shifting later content, a clip's linked sound on another track), how a batch
of actions stays atomic and refers to items it creates, where changesets live and how they survive
a crash, and how a revert is triggered and reported.

## Decision
- **One edit service.** `se_app::AgentEditService::apply(agent, label, edits)` is the only way
  write tools change a project. se-mcp turns a tool's input into typed `AgentEdit`s (times in
  flicks, ids resolved, styles built: pure functions per tool) and the service plans them in order
  on a draft (`EditPlan`, so each edit sees the previous ones and a refusal names its edit), checks
  the scope, and commits **one** transaction of actor `AiTask(task)` through the project's actor.
  Created items carry `origin = AiTask(task)`.
- **Scope = everything touched lies inside, before and after.** After planning, the service
  applies the whole transaction to a draft and checks every touched entity: items (sequence,
  tracks, `range.contains_range(item)`, in both states), markers (time in the range), tracks (no
  header change, no removal; added only when the task may use every track), transitions (both
  items in scope), sequences (only the task's, never renamed or reformatted), never assets or the
  project. Items crossing the scope's edges cannot be changed, new items must lie inside. Because
  the check covers what an edit drags along, ripple deletes and inserted gaps are refused in a
  time-limited scope when they would shift later content outside it (strict rule chosen by the
  user; the message suggests `delete_items` or `trim_item`). Errors are `OUT_OF_SCOPE` messages
  naming the entity, its place, the scope and what to do.
- **Linked items move together.** Moves, trims, splits and deletes of agents follow the link group
  as the timeline does (user's choice): the partner moves by the same amount, trims when its edge
  sits at the same instant, is cut when it crosses the instant, is deleted with it. A partner
  outside the scope makes the edit `OUT_OF_SCOPE` rather than desynchronising picture and sound.
- **Optimistic concurrency.** Tools changing an item take `expected_revision` (the domain's
  `REVISION_CONFLICT`). The scope is checked on the state the call was planned on: the actor
  commits only if the project is still at that revision (`ProjectActorHandle::apply_ai_on`),
  otherwise the service plans again (up to four times), so a user edit landing in between can
  never make an unchecked change commit.
- **Batches.** `apply_batch` plans several tools' inputs into one call: one transaction, all or
  nothing, errors prefixed with the failing action. An action names the item it creates
  (`"as": "title"`); later actions write `"$title"`, resolved to `ItemRef::Created { edit }` and
  then to the id the planner gave that item.
- **Changesets live in the project's actor and in `project.db`.** The actor keeps the open
  changeset of each task (loaded at opening). An AI commit appends its entry
  (`Changeset::record`: transaction, inverse, revisions left) and writes journal entry and
  changeset entry **in one SQLite transaction** (`ProjectStore::append_journal_with_changeset`;
  schema v4: `changesets`, `changeset_entries`), so a crash never separates them; it then publishes
  `projectPatched` and `taskChangesetUpdated { taskId, changesetId, touchedItems }`. The first edit
  creates the changeset; the task records its id (`Task.changesetId`, `taskUpdated`). One changeset
  per task, across its runs.
- **Revert.** Protocol command `revertTask { projectId, taskId }` → `TaskService::revert` (refused
  while the task is queued or running: `TASK_STILL_ACTIVE`) → the actor runs `plan_revert` on the
  current document and commits the plan as one **user** transaction labelled « Revert AI task »
  (undoable), marking the changeset reverted in the same SQLite transaction. The answer is a
  `TaskRevertReport { reverted, revertedItems, conflicts }`; a plan with nothing to undo changes
  nothing and keeps the changeset (`reverted: false`); a task without an open changeset gets
  `NOTHING_TO_REVERT`. After a revert the task's `changesetId` is `null` and later edits (a
  follow-up) open a new changeset.
- **Tool error codes.** Domain refusals are mapped for agents: `REVISION_CONFLICT`,
  `LOCKED_TRACK`, `*_NOT_FOUND`, otherwise `VALIDATION_ERROR` with the domain rule's code in the
  message; `InvalidEdit` is `VALIDATION_ERROR` too; other use-case errors keep their protocol code
  (`OUT_OF_SCOPE`, `TRANSITION_NO_ROOM`, `RIPPLE_BLOCKED`…).

## Consequences
- An agent can build a complete, editable result in one call (`add_text` with its animations, or
  a batch), and the user can revert a whole task at any time, keeping their own later changes.
- Scope violations are impossible to commit, whatever an edit drags along, and agents get
  messages they can act on; the price is that ripple edits need a scope reaching the end of the
  sequence.
- Every agent call costs a few full applies of the transaction (planning per edit, the scope
  draft, the commit): fine for agent-sized calls (≤ 50 actions).
- A revert restores fields of tracks, sequences and markers without per-entity revisions
  (ADR-0016's limitation, unchanged); agents cannot change those anyway, except markers they add.
- Changesets are never deleted: reverted ones stay in `project.db` (cheap, useful for diagnostics
  and a future history view).

## Alternatives considered
- **Scope checks inside each tool**: every tool would re-implement the rule and miss what edits
  drag along (ripple shifts, partners, transitions). Rejected for one check on the planned
  transaction.
- **Tolerating time shifts of out-of-scope items in ripple edits**: simpler for agents, but a
  task scoped to 51–54 s could move the rest of the film. Offered to the user, rejected.
- **Editing only the listed items (no link following)**: agents would desynchronise picture and
  sound unless they listed partners themselves. Offered to the user, rejected.
- **Changesets in se-app memory or in the task service, saved separately**: a crash between the
  journal and the changeset write would leave a committed AI edit outside its changeset. Rejected
  for one atomic write inside the actor that commits it.
- **Checking the scope inside the actor**: the actor would need task scopes and the planner's
  context; the revision guard gives the same guarantee with the actor unchanged in spirit.

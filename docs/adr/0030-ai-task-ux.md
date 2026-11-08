# 0030 — AI task UX: composer scopes, a renderer task mirror, patch actors and revert previews

- Status: Accepted
- Date: 2026-11-08

## Context
Prompt 20 turns the AI runtime (ADR-0027) and the agents' write tools (ADR-0029) into the
editor's central experience: pick a passage (or write « de 51 s à 54 s… »), send a prompt, keep
editing while several tasks run, watch each one live, then keep, iterate on or revert its result.
This needs decisions on where the prompt's scope comes from, how the timeline learns about tasks
without depending on the AI feature, how « Refaire avec l'IA » resumes a task that may no longer
cover the item, how a revert's conflicts are shown before it happens, and how the UI tells AI
edits from the user's.

## Decision
- **Composer scope, by priority**: a time range read in the prompt's text (a pure French parser,
  `findTimePhrase`, snapping to the sequence's frames; applied automatically, shown as a dashed
  chip, dismissable), then the item of « Refaire avec l'IA », then the selected items (bounding
  range and their tracks), then the in/out span, else the whole sequence. Tracks come from the
  item or selection unless the user chooses them (or « toutes les pistes »); the sequence is the
  active one unless chosen. Removing the automatic range chip means « toute la séquence ».
- **One renderer mirror of tasks, in `shared/ai-tasks`**: tasks of the open project with their
  numbers (« #3 », oldest first — tasks are never deleted), items to pulse, running tasks whose
  scope the user edited, and « Voir » / composer requests. The AI feature's sync
  (`connectTaskSync`, mounted by the editor layout) feeds it; the timeline only reads it, so
  features keep importing each other through `index.ts` only and in one direction.
- **`projectPatched` names its actor** (`ProjectPatch.actor`: user, AI task, system, from the
  committed transaction). The renderer pulses items of AI patches and flags running tasks whose
  scope a user patch touched (the agent may then meet `REVISION_CONFLICT`); editing is never
  blocked.
- **Follow-ups may change the task's scope** (`followUpTask { scope? }`, same sequence only): the
  task keeps the scope of its latest run, and each run's MCP token carries it. « Refaire avec
  l'IA » widens the original scope to the item (bounding range, union of tracks) when the item
  moved out, so the resumed agent can still edit it.
- **Revert previews**: `previewTaskRevert` plans and commits the revert on a copy inside the actor
  and reports what it would undo and keep; the panel lists conflicts before the user confirms,
  and reverts at once (undoable) when there are none. `getTaskChanges` lists a task's surviving
  items (« Voir les modifications »).
- **`Task.usage`** adds up the `usage` events of a task's runs (tokens, cost when reported), so
  cards and the panel total need no event logs.
- **Tool names are provider-agnostic in events**: the Claude Code provider reports our MCP tools
  by their own name (`add_text`, not `mcp__simpleeffect__add_text`), and keeps 1000 characters of
  inputs and outputs so the log shows compact lines (« add_text “Abonne-toi” 00:51→00:54 ») and
  tool lines can select the items they name.
- **Notifications**: a toast with « Voir » at the end of each run; a system notification through
  the bridge (`showNotification`, `onNotificationClick`) when the window is not focused.

## Consequences
- The timeline draws scopes, pulses and origin badges without knowing the AI feature; new
  task-aware UI reads the same mirror.
- Patches are slightly larger (one small actor object); clients may now attribute any change.
- A task's scope can grow across follow-ups: its overlay and any later revert follow the latest
  scope, and the changeset still holds every run's edits.
- A revert preview costs one extra planning pass (pure, on a copy) before reverts with
  conflicts.
- The French time parser is a heuristic: three-part clocks are hours (`1:02:03`), four-part ones
  SMPTE at the sequence's rate, bare numbers only borrow a unit from the other bound. Other
  languages will need their own phrases.

## Alternatives considered
- **Timeline overlays injected by the editor composition (render props)**: the item-level parts
  (pulse, origin badge, menu entry) would need several slots threaded through the timeline's
  components. Rejected for a shared read-only mirror.
- **Inferring user edits from `historyChanged`**: AI commits publish it too, and ordering with
  `taskChangesetUpdated` is fragile. Rejected for an explicit patch actor.
- **Pulsing from `taskChangesetUpdated.touchedItems`**: the list is the whole changeset, so every
  edit would pulse every earlier item again. Rejected.
- **Reverting first and listing conflicts afterwards**: the prompt asks for the conflicts in the
  dialog; a preview lets the user keep everything instead.
- **A new task for every « Refaire avec l'IA »**: loses the agent's context of what it did; kept
  as an option (untick « reprendre la session »).

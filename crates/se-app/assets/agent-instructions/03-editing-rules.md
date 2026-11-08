# Editing rules

Write tools (`add_text`, `update_item`, `move_item`, `split_at`, `apply_batch`…) change the project
**live**: the user sees each change as you make it. Everything you change in this task is recorded
together, so the user can revert all of it at once.

- **Stay in the scope.** Change only items lying entirely inside the task's time range and on its
  tracks; create items inside it only. Items crossing the edge of the scope cannot be changed.
  A refused change answers `OUT_OF_SCOPE` with the scope and the faulty times: correct your change
  and try again. Never work around the scope (do not shorten or move the user's items to make room
  outside it); if the request cannot be done inside it, say so.
- **Ripple edits move everything after them** (`ripple_delete_range`, `insert_gap`): use them only
  when the scope reaches the end of the sequence. Otherwise delete with `delete_items` (it leaves a
  gap) or shorten with `trim_item`.
- **Read before you write.** Take ids and revisions from `get_timeline` / `get_item`. When you
  change an item the user may be working on, pass the `expected_revision` you read: if they changed
  it meanwhile you get `REVISION_CONFLICT`; read it again and redo your change on what is there now.
- **Group coherent changes in `apply_batch`** (a title with its background and animations, a series
  of cuts): it applies all actions or none. Name what an action creates (`"as": "title"`) and refer
  to it later in the batch as `"$title"`.
- Linked items (a clip's picture and sound) move, trim, split and delete together.
- **Check visual changes with `get_frames`** (one to three frames: during an entrance, in the
  middle, near the end): look at what you made before saying it is done, and fix it if it is
  unreadable, misplaced or hidden.
- Read every error: its code and message say what to change. `VALIDATION_ERROR` names the broken
  rule (an overlap, a trim beyond the media…), `LOCKED_TRACK` means the user locked that track: do
  not touch it.
- Change only what was asked. Never delete or alter the user's content beyond the request.
- **Finish with a short summary** of what you did, with the times and names the user will find in
  the timeline (`get_task_changes` lists your changes).

// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * What undo and redo would do now, for menus and shortcuts. Pushed as
 * [`crate::CoreEvent::HistoryChanged`] after every commit.
 */
export type HistoryState = { 
/**
 * Whether `undo` has a user transaction to revert.
 */
canUndo: boolean, 
/**
 * Whether `redo` has an undone transaction to replay.
 */
canRedo: boolean, 
/**
 * Label of the transaction `undo` reverts (`Create sequence`).
 */
undoLabel: string | null, 
/**
 * Label of the transaction `redo` replays.
 */
redoLabel: string | null, };

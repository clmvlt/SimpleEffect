// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { HistoryState } from "./HistoryState";
import type { ProjectId } from "./ProjectId";

/**
 * [`HistoryState`] of one project, as pushed by [`crate::CoreEvent::HistoryChanged`].
 */
export type ProjectHistory = { 
/**
 * The project whose history changed.
 */
projectId: ProjectId, 
/**
 * Its new undo state.
 */
history: HistoryState, };

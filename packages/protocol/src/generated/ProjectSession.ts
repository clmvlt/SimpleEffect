// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { HistoryState } from "./HistoryState";
import type { ProjectRecovery } from "./ProjectRecovery";
import type { ProjectSnapshot } from "./ProjectSnapshot";

/**
 * An open project: where it lives, its entities and its undo state. Answers `createProject`,
 * `openProject` and `getProjectSnapshot`, and is pushed as [`crate::CoreEvent::ProjectOpened`].
 */
export type ProjectSession = { 
/**
 * Absolute path of the project folder (`…/<Name>.seproj`).
 */
path: string, 
/**
 * Every entity at the current revision.
 */
snapshot: ProjectSnapshot, 
/**
 * What undo and redo would do now.
 */
history: HistoryState, 
/**
 * How opening recovered the project after an unexpected stop (prompt 29); `null` when it
 * had been closed cleanly. Kept for as long as the project stays open.
 */
recovery: ProjectRecovery | null, };

// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { HistoryState } from "./HistoryState";
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
history: HistoryState, };

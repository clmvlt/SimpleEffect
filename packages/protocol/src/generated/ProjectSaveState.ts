// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ProjectId } from "./ProjectId";

/**
 * Whether an open project's snapshot can be saved, as pushed by
 * [`crate::CoreEvent::ProjectSaveStateChanged`].
 */
export type ProjectSaveState = { 
/**
 * The project.
 */
projectId: ProjectId, 
/**
 * Why the last save failed; `null` once a save succeeded again.
 */
failure: string | null, 
/**
 * Changes not in `project.json` yet (kept by the journal).
 */
unsavedChanges: number, };

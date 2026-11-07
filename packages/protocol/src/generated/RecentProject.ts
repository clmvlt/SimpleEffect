// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Format } from "./Format";
import type { ProjectId } from "./ProjectId";

/**
 * A project of the home screen's recent list.
 */
export type RecentProject = { 
/**
 * Stable id of the project.
 */
id: ProjectId, 
/**
 * Its name.
 */
name: string, 
/**
 * Absolute path of its folder.
 */
path: string, 
/**
 * Its default sequence format (shown as a badge: 16:9, 9:16…).
 */
format: Format, 
/**
 * When it was last opened (or created), in milliseconds since the Unix epoch.
 */
lastOpenedAt: number, 
/**
 * Whether the folder is still there (it may have been moved or deleted outside the app).
 */
available: boolean, };

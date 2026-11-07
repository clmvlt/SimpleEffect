// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ProjectId } from "./ProjectId";

/**
 * Arguments of [`crate::Command::ImportMedia`].
 */
export type ImportMediaRequest = { 
/**
 * The open project receiving the media.
 */
projectId: ProjectId, 
/**
 * Absolute paths of the files, in the order they should appear in the media bin.
 */
paths: Array<string>, };

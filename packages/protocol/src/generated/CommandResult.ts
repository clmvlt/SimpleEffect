// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { CoreInfo } from "./CoreInfo";
import type { NewProjectDefaults } from "./NewProjectDefaults";
import type { ProjectSession } from "./ProjectSession";
import type { RecentProject } from "./RecentProject";
import type { SequenceId } from "./SequenceId";

/**
 * The result of a [`crate::Command`], tagged by the **same `kind` as the command** it answers.
 */
export type CommandResult = { "kind": "ping" } | { "kind": "getCoreInfo" } & CoreInfo | { "kind": "getNewProjectDefaults" } & NewProjectDefaults | { "kind": "createProject" } & ProjectSession | { "kind": "openProject" } & ProjectSession | { "kind": "closeProject" } | { "kind": "listRecentProjects", 
/**
 * Most recently opened first.
 */
projects: Array<RecentProject>, } | { "kind": "renameProject" } & RecentProject | { "kind": "duplicateProject" } & RecentProject | { "kind": "trashProject" } | { "kind": "getProjectSnapshot" } & ProjectSession | { "kind": "createSequence", 
/**
 * Id of the new sequence.
 */
sequenceId: SequenceId, } | { "kind": "undo" } | { "kind": "redo" };

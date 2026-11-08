// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Actor } from "./Actor";
import type { EntityDeletes } from "./EntityDeletes";
import type { EntityUpserts } from "./EntityUpserts";
import type { ProjectId } from "./ProjectId";

/**
 * What one commit changed: entities to put in place (whole, replacing any with the same id) and
 * ids of entities that are gone. Carried by [`crate::CoreEvent::ProjectPatched`].
 */
export type ProjectPatch = { 
/**
 * The project the commit belongs to.
 */
projectId: ProjectId, 
/**
 * Revision after the commit. A client whose revision is not `revision − 1` missed a patch
 * and must reload the snapshot (`getProjectSnapshot`).
 */
revision: number, 
/**
 * Who made the commit: the user (edits, undo, redo, reverts), an AI task, or the app.
 */
actor: Actor, 
/**
 * Entities added or changed.
 */
upserts: EntityUpserts, 
/**
 * Entities removed.
 */
deletes: EntityDeletes, };

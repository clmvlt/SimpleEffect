// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ChangesetId } from "./ChangesetId";
import type { EffortLevel } from "./EffortLevel";
import type { ProjectId } from "./ProjectId";
import type { ProviderId } from "./ProviderId";
import type { TaskId } from "./TaskId";
import type { TaskRun } from "./TaskRun";
import type { TaskScope } from "./TaskScope";
import type { TaskStatus } from "./TaskStatus";

/**
 * An AI task as clients see it (its events come apart: `getTaskEvents`, `taskEventAppended`).
 */
export type Task = { 
/**
 * Stable id.
 */
id: TaskId, 
/**
 * The project it belongs to.
 */
projectId: ProjectId, 
/**
 * The user's request (the first run's prompt).
 */
prompt: string, 
/**
 * What it may change.
 */
scope: TaskScope, 
/**
 * The provider running it.
 */
provider: ProviderId, 
/**
 * The model asked for; `null` lets the CLI decide.
 */
model: string | null, 
/**
 * The effort asked for; `null` lets the CLI decide.
 */
effort: EffortLevel | null, 
/**
 * Status of its latest run.
 */
status: TaskStatus, 
/**
 * The agent session its runs share, once the CLI announced it.
 */
agentSessionId: string | null, 
/**
 * When it was submitted (milliseconds since the Unix epoch).
 */
createdAt: number, 
/**
 * When it last changed.
 */
updatedAt: number, 
/**
 * Its runs, oldest first (never empty).
 */
runs: Array<TaskRun>, 
/**
 * The changeset holding its edits since it started or was last reverted (`null` while it
 * changed nothing). Created by its first edit (prompt 19).
 */
changesetId: ChangesetId | null, };

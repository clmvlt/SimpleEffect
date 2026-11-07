// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AiStatus } from "./AiStatus";
import type { JobProgress } from "./JobProgress";
import type { JobSnapshot } from "./JobSnapshot";
import type { ProjectHistory } from "./ProjectHistory";
import type { ProjectId } from "./ProjectId";
import type { ProjectPatch } from "./ProjectPatch";
import type { ProjectSession } from "./ProjectSession";
import type { Task } from "./Task";
import type { TaskEventRecord } from "./TaskEventRecord";
import type { TaskId } from "./TaskId";

/**
 * An event pushed inside a [`crate::EventEnvelope`], tagged by `kind` (camelCase).
 *
 * Named `CoreEvent` rather than `Event` so the generated TypeScript type does not shadow the DOM
 * `Event` type in the renderer.
 */
export type CoreEvent = { "kind": "coreReady", 
/**
 * Version of the running `se-core` binary.
 */
version: string, } | { "kind": "coreShuttingDown" } | { "kind": "resyncRequired", 
/**
 * Number of events that were dropped.
 */
missedEvents: number, } | { "kind": "projectOpened" } & ProjectSession | { "kind": "projectPatched" } & ProjectPatch | { "kind": "projectClosed", 
/**
 * The project.
 */
projectId: ProjectId, } | { "kind": "historyChanged" } & ProjectHistory | { "kind": "jobQueued", 
/**
 * The job.
 */
job: JobSnapshot, } | { "kind": "jobProgress" } & JobProgress | { "kind": "jobCompleted", 
/**
 * The job.
 */
job: JobSnapshot, } | { "kind": "jobFailed", 
/**
 * The job.
 */
job: JobSnapshot, } | { "kind": "jobCancelled", 
/**
 * The job.
 */
job: JobSnapshot, } | { "kind": "taskUpdated", 
/**
 * The task as it is now.
 */
task: Task, } | { "kind": "taskEventAppended", 
/**
 * The task's project.
 */
projectId: ProjectId, 
/**
 * The task.
 */
taskId: TaskId, 
/**
 * The new entry of its log.
 */
event: TaskEventRecord, } | { "kind": "aiStatusChanged" } & AiStatus;

// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { EffortLevel } from "./EffortLevel";
import type { ProjectId } from "./ProjectId";
import type { ProviderId } from "./ProviderId";
import type { TaskScope } from "./TaskScope";

/**
 * Arguments of [`crate::Command::SubmitTask`].
 */
export type SubmitTaskRequest = { 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * What the agent must do.
 */
prompt: string, 
/**
 * What it may change.
 */
scope: TaskScope, 
/**
 * The provider; omitted for the one chosen in the settings.
 */
provider?: ProviderId, 
/**
 * The model; omitted for the settings' default.
 */
model?: string, 
/**
 * The effort; omitted for the settings' default.
 */
effort?: EffortLevel, };

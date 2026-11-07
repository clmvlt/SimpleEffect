// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { EffortLevel } from "./EffortLevel";
import type { ProviderId } from "./ProviderId";

/**
 * AI settings of the app (`library.db`, edited in « Réglages » → IA).
 */
export type AiSettings = { 
/**
 * The provider running tasks.
 */
provider: ProviderId, 
/**
 * Absolute path of the CLI executable chosen by the user; `null` searches it.
 */
executablePath: string | null, 
/**
 * Model of new tasks (a [`ModelOption::id`]); `null` lets the CLI decide.
 */
defaultModel: string | null, 
/**
 * Effort of new tasks; `null` lets the CLI decide.
 */
defaultEffort: EffortLevel | null, 
/**
 * Tasks running at once, in [`AiSettings::MAX_CONCURRENT_TASKS_RANGE`]; others wait.
 */
maxConcurrentTasks: number, };

// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { BrandKit } from "./BrandKit";
import type { MotionTemplate } from "./MotionTemplate";
import type { ProjectId } from "./ProjectId";
import type { ProjectSettings } from "./ProjectSettings";

/**
 * The fields of a project that are not entities of their own.
 */
export type ProjectHeader = { 
/**
 * Stable id.
 */
id: ProjectId, 
/**
 * Name shown in the UI.
 */
name: string, 
/**
 * Project-wide preferences.
 */
settings: ProjectSettings, 
/**
 * Motion templates available to template items.
 */
templates: Array<MotionTemplate>, 
/**
 * The brand identity of the project, if one was set.
 */
brandKit: BrandKit | null, };

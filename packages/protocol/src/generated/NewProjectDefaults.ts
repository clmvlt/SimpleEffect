// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Format } from "./Format";
import type { FormatPresetDimensions } from "./FormatPresetDimensions";
import type { FrameRate } from "./FrameRate";

/**
 * Choices offered by the « new project » dialog, so the renderer never duplicates domain lists.
 */
export type NewProjectDefaults = { 
/**
 * Folder proposed for the new project: the last one used, or `<Documents>/SimpleEffect`.
 */
parentDir: string, 
/**
 * Format proposed for the new project.
 */
format: Format, 
/**
 * Named format presets and their dimensions.
 */
presets: Array<FormatPresetDimensions>, 
/**
 * Frame rates the domain supports exactly, slowest first.
 */
frameRates: Array<FrameRate>, };

// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ExportPreset } from "./ExportPreset";

/**
 * What the export dialog remembers between exports (the app's library).
 */
export type ExportPreferences = { 
/**
 * The folder exports are written to (the last one used; the user's videos folder at first).
 */
directory: string, 
/**
 * The file name template (`{project}`, `{sequence}`, `{preset}`, `{date}`, `{time}`,
 * `{width}`, `{height}`).
 */
fileNameTemplate: string, 
/**
 * The user's saved presets.
 */
presets: Array<ExportPreset>, };

// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ExportSettings } from "./ExportSettings";
import type { ProjectId } from "./ProjectId";
import type { SequenceId } from "./SequenceId";
import type { TimeRange } from "./TimeRange";

/**
 * An export to start.
 */
export type StartExportRequest = { 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The sequence to export.
 */
sequenceId: SequenceId, 
/**
 * What to write.
 */
settings: ExportSettings, 
/**
 * The part of the sequence (its in/out span), or the whole sequence when absent.
 */
range?: TimeRange, 
/**
 * Absolute path of the file to write (replaced when it exists); its extension must match the
 * container.
 */
path: string, 
/**
 * Name of the preset, shown in the exports list.
 */
presetName: string, };

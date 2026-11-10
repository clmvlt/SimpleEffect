// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AudioExportSettings } from "./AudioExportSettings";
import type { ExportContainer } from "./ExportContainer";
import type { ExportPresetVideo } from "./ExportPresetVideo";

/**
 * Settings the user saved under a name (« Réglages perso »).
 */
export type ExportPreset = { 
/**
 * Stable id (chosen by the renderer).
 */
id: string, 
/**
 * Name shown in the preset list.
 */
name: string, 
/**
 * The file's container.
 */
container: ExportContainer, 
/**
 * The picture, or `null` for sound only.
 */
video: ExportPresetVideo | null, 
/**
 * The sound, or `null` for silence.
 */
audio: AudioExportSettings | null, };

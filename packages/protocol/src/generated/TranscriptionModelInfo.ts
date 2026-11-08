// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { DownloadProgress } from "./DownloadProgress";

/**
 * A transcription model: one of the catalogue (installed or not) or a file the user imported.
 */
export type TranscriptionModelInfo = { 
/**
 * Stable id: the catalogue's (`tiny`, `base`, `small`, `medium`, `large-v3-turbo`) or, for an
 * imported model, its file name.
 */
id: string, 
/**
 * The file in the models folder.
 */
fileName: string, 
/**
 * Its size in bytes (expected for models not installed).
 */
size: number, 
/**
 * Whether the file is in the models folder.
 */
installed: boolean, 
/**
 * Whether it was imported (not one of the catalogue).
 */
custom: boolean, 
/**
 * Whether it is the catalogue model suggested for this machine.
 */
recommended: boolean, 
/**
 * Its download, while one runs or after one failed.
 */
download: DownloadProgress | null, };

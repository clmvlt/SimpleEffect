// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { DownloadProgress } from "./DownloadProgress";

/**
 * The voice detection model (one file, downloaded with the first transcription model).
 */
export type VoiceModelInfo = { 
/**
 * Its size in bytes.
 */
size: number, 
/**
 * Whether it is installed.
 */
installed: boolean, 
/**
 * Its download, while one runs or after one failed.
 */
download: DownloadProgress | null, };

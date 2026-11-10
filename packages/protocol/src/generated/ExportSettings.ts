// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AudioExportSettings } from "./AudioExportSettings";
import type { ExportContainer } from "./ExportContainer";
import type { VideoExportSettings } from "./VideoExportSettings";

/**
 * What an export contains and how it is encoded.
 */
export type ExportSettings = { 
/**
 * The file's container.
 */
container: ExportContainer, 
/**
 * The picture, or `null` for a sound-only file.
 */
video: VideoExportSettings | null, 
/**
 * The sound, or `null` for a silent file.
 */
audio: AudioExportSettings | null, };

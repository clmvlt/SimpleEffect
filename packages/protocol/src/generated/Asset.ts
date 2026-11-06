// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { AssetKind } from "./AssetKind";
import type { Decodability } from "./Decodability";
import type { DerivedMedia } from "./DerivedMedia";
import type { MediaInfo } from "./MediaInfo";

/**
 * A video, audio or image file used by the project. The file is referenced, never copied.
 */
export type Asset = { 
/**
 * Stable id.
 */
id: AssetId, 
/**
 * Absolute path of the user's file, in the native form of the OS that imported it.
 */
originalPath: string, 
/**
 * Path relative to the project folder, `/`-separated, used to relink the file when the
 * project and its media move together. `None` when the file is on another drive.
 */
relativeHint: string | null, 
/**
 * What the file contains; decides which items may use it.
 */
kind: AssetKind, 
/**
 * Probed facts (duration, streams); `None` until the probe ran (prompt 08).
 */
mediaInfo: MediaInfo | null, 
/**
 * Mezzanine and proxy copies generated for this asset.
 */
derived: DerivedMedia, 
/**
 * Whether the renderer can decode the original with WebCodecs.
 */
decodability: Decodability, 
/**
 * The original was not found at its path or relative hint when the project was opened.
 */
missing: boolean, };

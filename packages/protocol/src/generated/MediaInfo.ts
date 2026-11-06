// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AudioStreamInfo } from "./AudioStreamInfo";
import type { Flicks } from "./Flicks";
import type { VideoStreamInfo } from "./VideoStreamInfo";

/**
 * Facts about a media file, filled from `ffprobe` (prompt 08).
 */
export type MediaInfo = { 
/**
 * Playable length; `None` for still images and when the container does not say.
 */
duration: Flicks | null, 
/**
 * Container format as reported by the prober (e.g. `mov,mp4,m4a,3gp,3g2,mj2`).
 */
container: string, 
/**
 * The main picture stream, if any.
 */
videoStream: VideoStreamInfo | null, 
/**
 * Sound streams, in file order.
 */
audioStreams: Array<AudioStreamInfo>, };

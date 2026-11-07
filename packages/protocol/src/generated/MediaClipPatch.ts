// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { FadeShape } from "./FadeShape";
import type { Flicks } from "./Flicks";
import type { Reframe } from "./Reframe";

/**
 * Changes of a [`crate::MediaClip`].
 */
export type MediaClipPatch = { 
/**
 * Another video or audio asset.
 */
assetId: AssetId | null, 
/**
 * Another source start (a slip edit: the item stays, the media slides under it).
 */
sourceStart: Flicks | null, 
/**
 * Another playback speed (> 0); the item keeps its duration and uses more or less media.
 */
speed: number | null, 
/**
 * Another fade-in length.
 */
fadeIn: Flicks | null, 
/**
 * Another fade-out length.
 */
fadeOut: Flicks | null, 
/**
 * Another fade-in curve.
 */
fadeInShape: FadeShape | null, 
/**
 * Another fade-out curve.
 */
fadeOutShape: FadeShape | null, 
/**
 * Another way to fit the frame.
 */
reframe: Reframe | null, };

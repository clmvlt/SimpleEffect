// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { FadeShape } from "./FadeShape";
import type { Flicks } from "./Flicks";
import type { Reframe } from "./Reframe";

/**
 * Changes of a [`crate::MediaClip`] (clients may omit unchanged fields).
 */
export type MediaClipPatch = { 
/**
 * Another video or audio asset.
 */
assetId?: AssetId, 
/**
 * Another source start (a slip edit: the item stays, the media slides under it).
 */
sourceStart?: Flicks, 
/**
 * Another playback speed (> 0); the item keeps its duration and uses more or less media.
 */
speed?: number, 
/**
 * Another fade-in length.
 */
fadeIn?: Flicks, 
/**
 * Another fade-out length.
 */
fadeOut?: Flicks, 
/**
 * Another fade-in curve.
 */
fadeInShape?: FadeShape, 
/**
 * Another fade-out curve.
 */
fadeOutShape?: FadeShape, 
/**
 * Another way to fit the frame.
 */
reframe?: Reframe, };

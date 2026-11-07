// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Animatable } from "./Animatable";
import type { AssetId } from "./AssetId";
import type { FadeShape } from "./FadeShape";
import type { Flicks } from "./Flicks";
import type { Reframe } from "./Reframe";
import type { Transform2D } from "./Transform2D";

/**
 * Shows and/or plays part of a video or audio asset.
 *
 * On a video track it shows the picture; on an audio track it plays the sound (a video asset's
 * sound included). Opacity lives in [`Transform2D::opacity`].
 */
export type MediaClip = { 
/**
 * The asset played.
 */
assetId: AssetId, 
/**
 * Time in the asset played at the item's first instant (≥ 0).
 */
sourceStart: Flicks, 
/**
 * Playback speed (> 0): 2.0 plays twice as fast, so the clip uses twice as much source.
 */
speed: number, 
/**
 * Placement of the picture.
 */
transform: Transform2D, 
/**
 * Gain in decibels (0 = unchanged, within [`Self::MIN_VOLUME_DB`]..=[`Self::MAX_VOLUME_DB`]);
 * keyframes make the volume curve (automation).
 */
volumeDb: Animatable<number>, 
/**
 * Fade from black/silence at the start (≥ 0; fades together fit in the item).
 */
fadeIn: Flicks, 
/**
 * Fade to black/silence at the end (≥ 0).
 */
fadeOut: Flicks, 
/**
 * Curve of the fade-in. Linear when absent, so documents written before schema version 4
 * still read.
 */
fadeInShape: FadeShape, 
/**
 * Curve of the fade-out (linear when absent).
 */
fadeOutShape: FadeShape, 
/**
 * How the picture fits the sequence frame before the transform applies.
 */
reframe: Reframe, };

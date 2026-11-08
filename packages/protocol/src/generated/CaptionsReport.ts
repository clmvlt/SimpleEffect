// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { CaptionStyleId } from "./CaptionStyleId";
import type { ItemId } from "./ItemId";
import type { TrackId } from "./TrackId";

/**
 * What a caption generation or import did.
 */
export type CaptionsReport = { 
/**
 * The captions created, in time order (empty when no word was found: nothing changed).
 */
itemIds: Array<ItemId>, 
/**
 * The caption track (created when the sequence had none).
 */
trackId: TrackId | null, 
/**
 * The style of the new captions.
 */
styleId: CaptionStyleId | null, 
/**
 * Captions replaced.
 */
replaced: number, 
/**
 * Media of the range not transcribed yet (their speech got no caption).
 */
untranscribedAssets: Array<AssetId>, };

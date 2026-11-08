// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { Flicks } from "./Flicks";

/**
 * A transcript segment matching a search, in its asset's time.
 */
export type TranscriptSearchHit = { 
/**
 * The asset.
 */
assetId: AssetId, 
/**
 * The segment's index in its transcript.
 */
segment: number, 
/**
 * Its start in the asset.
 */
start: Flicks, 
/**
 * Its end in the asset.
 */
end: Flicks, 
/**
 * Its text.
 */
text: string, };

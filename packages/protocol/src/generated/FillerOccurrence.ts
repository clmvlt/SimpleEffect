// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { Flicks } from "./Flicks";
import type { ItemId } from "./ItemId";

/**
 * A filler word (or phrase) found in a sequence.
 */
export type FillerOccurrence = { 
/**
 * Its words as written.
 */
text: string, 
/**
 * Where it starts in the sequence.
 */
start: Flicks, 
/**
 * Where it ends in the sequence.
 */
end: Flicks, 
/**
 * The clip playing it.
 */
itemId: ItemId, 
/**
 * The asset it was said in.
 */
assetId: AssetId, };

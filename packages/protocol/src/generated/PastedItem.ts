// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Item } from "./Item";
import type { TrackId } from "./TrackId";

/**
 * An item given by content (`pasteItems`): the clipboard keeps copies because the originals
 * may be gone (cut, then pasted).
 */
export type PastedItem = { 
/**
 * The track receiving the copy.
 */
trackId: TrackId, 
/**
 * The content copied, already at its new start. Its id, revision and origin are replaced
 * (the copy is a new user item); items sharing a link group stay linked together in a new
 * group.
 */
item: Item, };

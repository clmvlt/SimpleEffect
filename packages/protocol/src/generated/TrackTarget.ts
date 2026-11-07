// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { TrackId } from "./TrackId";

/**
 * The track a part (picture or sound) of dropped media goes to.
 */
export type TrackTarget = { "kind": "existing", 
/**
 * The track.
 */
trackId: TrackId, } | { "kind": "new" };

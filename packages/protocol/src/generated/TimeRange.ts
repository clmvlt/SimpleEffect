// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";

/**
 * The half-open interval `[start, start + duration)`.
 *
 * Invariants, enforced at construction and deserialization: `duration ≥ 0` and the end stays
 * within the supported range. An empty range (`duration == 0`) contains and intersects nothing.
 * `start` may be negative (relative offsets); documents reject negative positions in validation.
 */
export type TimeRange = { start: Flicks, duration: Flicks, };

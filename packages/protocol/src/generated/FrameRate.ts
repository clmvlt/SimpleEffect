// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * Frames per second as a reduced fraction, e.g. `30000/1001` for 29.97 fps.
 *
 * Only rates whose frame lasts a whole number of flicks are representable (every standard rate
 * does), so frame positions convert to flicks exactly. The fraction is stored reduced: `60/2`
 * becomes `30/1`, and equal rates compare equal.
 */
export type FrameRate = { numerator: number, denominator: number, };

// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * An instant or a duration, in flicks: 1/705,600,000 of a second (ADR-0004).
 *
 * This base divides every common frame rate (NTSC x/1001 rates included) and sample rate, so
 * frame and sample positions are exact integers. Values are kept within ±[`Flicks::MAX_SAFE`],
 * which JavaScript numbers hold exactly: a `Flicks` is serialized as a plain JSON number and is a
 * branded `number` in TypeScript.
 *
 * Arithmetic is checked (`checked_add`…): an overflow is a value, never a panic. Negative values
 * are allowed (offsets, deltas); the validation rejects negative positions in a document.
 */
export type Flicks = number & { readonly __brand: 'Flicks' };

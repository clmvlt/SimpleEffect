// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * The invariant a [`Violation`] breaks. Serialized in `SCREAMING_SNAKE_CASE`.
 */
export type ViolationCode = "DUPLICATE_ID" | "UNKNOWN_REFERENCE" | "ASSET_KIND_MISMATCH" | "INVALID_FORMAT" | "NEGATIVE_TIME" | "TIME_OUT_OF_RANGE" | "EMPTY_ITEM" | "OVERLAPPING_ITEMS" | "ITEM_NOT_ALLOWED_ON_TRACK" | "INVALID_SPEED" | "SOURCE_RANGE_EXCEEDS_MEDIA" | "EMPTY_KEYFRAMES" | "KEYFRAMES_NOT_SORTED" | "KEYFRAME_OUTSIDE_ITEM" | "VALUE_OUT_OF_RANGE";

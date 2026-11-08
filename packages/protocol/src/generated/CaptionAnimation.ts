// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * How caption words animate while spoken. The active word is the last one whose start has
 * passed; it stays active until the next word starts.
 */
export type CaptionAnimation = "none" | "wordHighlight" | "popPerWord" | "boxPerLine" | "revealPerWord";

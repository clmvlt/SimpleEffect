// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ItemId } from "./ItemId";
import type { PropertyAnimation } from "./PropertyAnimation";

/**
 * A new value of one animatable property of one item (`setItemAnimations`).
 */
export type ItemAnimation = { 
/**
 * The item animated.
 */
itemId: ItemId, 
/**
 * The property and its new keyframes or static value.
 */
animation: PropertyAnimation, };

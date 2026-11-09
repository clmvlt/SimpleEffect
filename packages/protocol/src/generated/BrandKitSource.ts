// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { BrandKitId } from "./BrandKitId";

/**
 * Where a project's kit comes from.
 */
export type BrandKitSource = { 
/**
 * The kit of the app's library.
 */
kitId: BrandKitId, 
/**
 * Its version when it was copied (a newer library version is proposed, never imposed).
 */
version: number, };

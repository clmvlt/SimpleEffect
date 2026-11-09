// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";

/**
 * One logo of a kit.
 */
export type BrandLogo = { 
/**
 * Name shown in the interface (« Logo blanc »).
 */
name: string, 
/**
 * The image asset of the project.
 */
assetId: AssetId, };

// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { DerivedKind } from "./DerivedKind";
import type { ProjectId } from "./ProjectId";

/**
 * Arguments of [`crate::Command::GenerateDerivedMedia`].
 */
export type GenerateDerivedRequest = { 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The asset.
 */
assetId: AssetId, 
/**
 * What to generate.
 */
derived: DerivedKind, };

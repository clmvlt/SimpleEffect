// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { ProjectId } from "./ProjectId";

/**
 * Arguments of [`crate::Command::RelinkAsset`].
 */
export type RelinkAssetRequest = { 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The asset to point at another file.
 */
assetId: AssetId, 
/**
 * Absolute path of the replacement file.
 */
path: string, 
/**
 * Relink even when the file's content differs from the asset's fingerprint (the user
 * confirmed after a [`RelinkOutcome::ContentDiffers`] answer).
 */
force: boolean, };

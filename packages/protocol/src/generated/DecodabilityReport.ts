// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AssetId } from "./AssetId";
import type { ProjectId } from "./ProjectId";
import type { StreamDecodability } from "./StreamDecodability";

/**
 * Arguments of [`crate::Command::ReportDecodability`].
 */
export type DecodabilityReport = { 
/**
 * The open project.
 */
projectId: ProjectId, 
/**
 * The asset checked.
 */
assetId: AssetId, 
/**
 * Whether WebCodecs decodes its picture stream (`noStream` when it has none).
 */
video: StreamDecodability, 
/**
 * Whether WebCodecs decodes its first sound stream (`noStream` when it has none).
 */
audio: StreamDecodability, };

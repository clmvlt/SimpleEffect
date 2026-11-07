// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Format } from "./Format";

/**
 * Payload of [`crate::Command::CreateProject`].
 */
export type CreateProjectRequest = { 
/**
 * Name of the project (not empty); its folder is `<parentDir>/<name>.seproj`.
 */
name: string, 
/**
 * Absolute path of the folder receiving the project folder (created if missing).
 */
parentDir: string, 
/**
 * Format of the project's default sequence (and of new sequences).
 */
format: Format, };

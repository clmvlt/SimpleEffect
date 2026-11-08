// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { AuthStatus } from "./AuthStatus";
import type { InstallationStatus } from "./InstallationStatus";
import type { ProviderCapabilities } from "./ProviderCapabilities";
import type { ProviderInfo } from "./ProviderInfo";

/**
 * Everything clients show about the AI provider in use.
 */
export type AiStatus = { 
/**
 * The provider.
 */
provider: ProviderInfo, 
/**
 * Whether its CLI was found.
 */
installation: InstallationStatus, 
/**
 * Whether its CLI is logged in.
 */
auth: AuthStatus, 
/**
 * What it supports.
 */
capabilities: ProviderCapabilities, };

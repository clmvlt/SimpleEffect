// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ProviderId } from "./ProviderId";

/**
 * What clients show about a provider, and how a user installs it or logs in. Written by the
 * provider so the UI never names a CLI itself.
 */
export type ProviderInfo = { 
/**
 * Stable id.
 */
id: ProviderId, 
/**
 * Product name (`Claude Code`).
 */
name: string, 
/**
 * What a user types in a terminal to start the CLI (`claude`).
 */
command: string, 
/**
 * What a user types inside the CLI to log in (`/login`).
 */
loginCommand: string, 
/**
 * Web page explaining how to install the CLI (opened in the user's browser on request).
 */
helpUrl: string, };

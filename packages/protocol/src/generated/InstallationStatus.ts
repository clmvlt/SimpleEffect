// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * Whether the provider's CLI was found and runs.
 */
export type InstallationStatus = { "kind": "installed", 
/**
 * Absolute path of the executable in use.
 */
path: string, 
/**
 * Its version (`2.1.317`).
 */
version: string, } | { "kind": "notInstalled", 
/**
 * Where it was looked for and what to do (install it, or set its path).
 */
message: string, } | { "kind": "unusable", 
/**
 * Absolute path of the executable tried.
 */
path: string, 
/**
 * What went wrong.
 */
message: string, };

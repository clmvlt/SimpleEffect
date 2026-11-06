// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * Facts about the running core, shown in the UI and useful in bug reports.
 */
export type CoreInfo = { 
/**
 * Version of the running `se-core` binary (semver).
 */
version: string, 
/**
 * Operating system, as Rust's `std::env::consts::OS` (`windows`, `macos`, `linux`).
 */
os: string, 
/**
 * CPU architecture, as Rust's `std::env::consts::ARCH` (`x86_64`, `aarch64`).
 */
arch: string, 
/**
 * Absolute path of the app data folder (`SE_DATA_DIR`, set by the Electron app).
 */
appDataDir: string, 
/**
 * Absolute path of the folder holding the core's daily log files (`<app data>/logs`).
 */
logsDir: string, };

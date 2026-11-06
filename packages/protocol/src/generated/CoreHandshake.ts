// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * Connection details announced by `se-core` on stdout at startup, read by the Electron supervisor.
 *
 * Serialised as exactly one line: `{"port":<u16>,"token":"<hex>","version":"<semver>"}`.
 * Everything else `se-core` logs goes to stderr. Exported to TypeScript for the Electron
 * supervisor, which parses and validates the line.
 */
export type CoreHandshake = { 
/**
 * TCP port on 127.0.0.1 where `se-core` listens (chosen by the OS).
 */
port: number, 
/**
 * Session token (256 random bits, lowercase hex) every client must present.
 */
token: string, 
/**
 * Version of the running `se-core` binary.
 */
version: string, };

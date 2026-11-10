// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * The archive `exportDiagnostics` wrote.
 */
export type DiagnosticsSummary = { 
/**
 * Where it was written (absolute).
 */
path: string, 
/**
 * The files inside, in archive order (`report.json`, `logs/…`).
 */
files: Array<string>, 
/**
 * Its size in bytes.
 */
bytes: number, };

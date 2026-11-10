// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * What opening a project had to recover: commits replayed from the journal (the app or the
 * core stopped before saving them) and, when `project.json` was unreadable, the backup it was
 * restored from.
 */
export type ProjectRecovery = { 
/**
 * Changes replayed from the journal on top of the snapshot.
 */
replayedChanges: number, 
/**
 * The backup file (`project.json.bak.1`…) the document was restored from, when
 * `project.json` could not be read; that file was kept as `project.json.corrupted`.
 */
restoredFrom: string | null, 
/**
 * Why `project.json` could not be read.
 */
problem: string | null, 
/**
 * Journal changes that could not be replayed on the backup (lost).
 */
lostChanges: number, };

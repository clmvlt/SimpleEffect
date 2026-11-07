// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * A new value for a patch field that may itself be `null`: `{"value": null}` clears the field,
 * while a `null` patch field leaves it unchanged.
 */
export type Assign<T> = { 
/**
 * The new value.
 */
value: T, };

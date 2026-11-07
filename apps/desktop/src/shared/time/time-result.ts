/**
 * The outcome of a time conversion that can fail: a value, or an error code naming the Rust
 * variant in camelCase (the codes of `docs/test-vectors/*.json`).
 */
export type TimeResult<T, E extends string> =
  { readonly ok: true; readonly value: T } | { readonly ok: false; readonly error: E };

/** A successful {@link TimeResult}. */
export function success<T>(value: T): { readonly ok: true; readonly value: T } {
  return { ok: true, value };
}

/** A failed {@link TimeResult}. */
export function failure<E extends string>(error: E): { readonly ok: false; readonly error: E } {
  return { ok: false, error };
}

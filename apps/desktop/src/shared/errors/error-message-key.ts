import { CoreRequestError, type ErrorCode } from '@se/protocol';

/**
 * Every protocol error code. `ErrorCodesCovered` below fails to compile when a code is added in
 * Rust (and regenerated) without being listed here, and `fr.json` must translate each one under
 * `errors.<CODE>` (checked by a test).
 */
export const ERROR_CODES = [
  'INVALID_REQUEST',
  'UNKNOWN_COMMAND',
  'INTERNAL',
  'TIMEOUT',
  'CONNECTION_LOST',
  'CLIENT_CLOSED',
  'PROJECT_NOT_FOUND',
  'PROJECT_LOCKED',
  'SCHEMA_TOO_NEW',
  'PROJECT_CORRUPTED',
  'PROJECT_ALREADY_EXISTS',
  'PROJECT_NOT_OPEN',
  'STORAGE_FAILED',
  'EDIT_REJECTED',
  'NOTHING_TO_UNDO',
  'NOTHING_TO_REDO',
] as const satisfies readonly ErrorCode[];

/** `true` only when {@link ERROR_CODES} lists every `ErrorCode`. */
export type ErrorCodesCovered =
  Exclude<ErrorCode, (typeof ERROR_CODES)[number]> extends never ? true : false;

/** Translation key of an error message. */
export type ErrorMessageKey = `errors.${ErrorCode}` | 'errors.unknown';

/** The translation key explaining `error` to the user: one per protocol code. */
export function errorMessageKey(error: unknown): ErrorMessageKey {
  return error instanceof CoreRequestError ? `errors.${error.code}` : 'errors.unknown';
}

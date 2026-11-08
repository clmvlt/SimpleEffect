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
  'ASSET_NOT_FOUND',
  'MEDIA_NOT_FOUND',
  'MEDIA_UNSUPPORTED',
  'MEDIA_KIND_MISMATCH',
  'MEDIA_TOOLS_MISSING',
  'JOB_NOT_FOUND',
  'DERIVED_NOT_APPLICABLE',
  'MEDIA_TIME_OUT_OF_RANGE',
  'RIPPLE_BLOCKED',
  'TRANSITION_NO_ROOM',
  'TASK_NOT_FOUND',
  'TASK_NOT_ACTIVE',
  'TASK_STILL_ACTIVE',
  'TASK_NOT_RESUMABLE',
  'AI_UNAVAILABLE',
  'RENDERER_UNAVAILABLE',
  'RENDER_FAILED',
  'RENDER_REQUEST_NOT_FOUND',
] as const satisfies readonly ErrorCode[];

/**
 * Domain codes (`details.domainCode` of `EDIT_REJECTED`) common enough in timeline edits to get
 * their own message (`errors.edit.<CODE>`); others use the generic `errors.EDIT_REJECTED`.
 */
export const EDIT_DETAIL_CODES = [
  'OVERLAPPING_ITEMS',
  'TRACK_LOCKED',
  'TRIM_BEYOND_SOURCE',
  'INVALID_EDIT_POINT',
  'FRAME_RATE_LOCKED',
  'LAST_SEQUENCE',
] as const;

/** A domain code with its own message. */
export type EditDetailCode = (typeof EDIT_DETAIL_CODES)[number];

/** `true` only when {@link ERROR_CODES} lists every `ErrorCode`. */
export type ErrorCodesCovered =
  Exclude<ErrorCode, (typeof ERROR_CODES)[number]> extends never ? true : false;

/** Translation key of an error message. */
export type ErrorMessageKey =
  `errors.${ErrorCode}` | `errors.edit.${EditDetailCode}` | 'errors.unknown';

/** The domain code of a refused edit, when it has its own message. */
function editDetailCode(error: CoreRequestError): EditDetailCode | null {
  const details = error.details;
  if (error.code !== 'EDIT_REJECTED' || typeof details !== 'object' || details === null) {
    return null;
  }
  const code = Array.isArray(details) ? undefined : details['domainCode'];
  return EDIT_DETAIL_CODES.find((known) => known === code) ?? null;
}

/**
 * The translation key explaining `error` to the user: one per protocol code, and a precise one
 * for the most common refused timeline edits (overlap, locked track…).
 */
export function errorMessageKey(error: unknown): ErrorMessageKey {
  if (!(error instanceof CoreRequestError)) return 'errors.unknown';
  const detail = editDetailCode(error);
  return detail === null ? `errors.${error.code}` : `errors.edit.${detail}`;
}

/** Why a picture (or a sound) could not be decoded. */
export type DecodeFailure =
  'noVideoTrack' | 'noAudioTrack' | 'codecNotDecodable' | 'readFailed' | 'imageNotDecodable';

/**
 * A decoding failure with an actionable message (read by users in the viewer and by agents).
 * The message never contains the media URL: it carries the session token.
 */
export class DecodeError extends Error {
  constructor(
    readonly failure: DecodeFailure,
    message: string,
    options?: { readonly cause?: unknown },
  ) {
    super(message, options);
    this.name = 'DecodeError';
  }
}

/** The error of a failed read, as a {@link DecodeError}. */
export function asDecodeError(
  error: unknown,
  failure: DecodeFailure,
  message: string,
): DecodeError {
  return error instanceof DecodeError ? error : new DecodeError(failure, message, { cause: error });
}

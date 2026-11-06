import type { ErrorCode, JsonValue, ProtocolError } from './generated';
import type { CommandKind } from './protocol-types';

/**
 * Rejection of {@link CoreClient.send}: an error answered by se-core, or a client-side failure
 * (`TIMEOUT`, `CONNECTION_LOST`, `CLIENT_CLOSED`). Switch on `code`, never on `message`.
 */
export class CoreRequestError extends Error {
  /** Stable error code shared with se-core (`ErrorCode`). */
  readonly code: ErrorCode;
  /** Kind of the command that failed. */
  readonly command: CommandKind;
  /** Structured context sent by se-core, if any. */
  readonly details: JsonValue | undefined;

  constructor(command: CommandKind, error: ProtocolError) {
    super(`${command} failed (${error.code}): ${error.message}`);
    this.name = 'CoreRequestError';
    this.code = error.code;
    this.command = command;
    this.details = error.details;
  }
}

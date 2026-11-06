/** Connection details printed by se-core as the first line of its stdout. */
export interface CoreHandshake {
  readonly port: number;
  readonly token: string;
  readonly version: string;
}

/** Result of {@link parseHandshakeLine}: errors are values, with an actionable message. */
export type HandshakeParseResult =
  | { readonly ok: true; readonly handshake: CoreHandshake }
  | { readonly ok: false; readonly error: string };

/** A session token is 256 random bits encoded as lowercase hex. */
const TOKEN_PATTERN = /^[0-9a-f]{64}$/;
const MAX_PORT = 65_535;

function failure(error: string): HandshakeParseResult {
  return { ok: false, error: `Invalid se-core handshake: ${error}` };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

/**
 * Parses the handshake line `{"port":<u16>,"token":"<hex>","version":"<semver>"}`.
 *
 * Error messages never echo the line: it may contain the session token.
 */
export function parseHandshakeLine(line: string): HandshakeParseResult {
  let value: unknown;
  try {
    value = JSON.parse(line.trim());
  } catch {
    return failure('the first stdout line is not JSON (did something print before the handshake?)');
  }
  if (!isRecord(value)) {
    return failure('expected a JSON object with "port", "token" and "version"');
  }
  const { port, token, version } = value;
  if (typeof port !== 'number' || !Number.isInteger(port) || port < 1 || port > MAX_PORT) {
    return failure('"port" must be an integer between 1 and 65535');
  }
  if (typeof token !== 'string' || !TOKEN_PATTERN.test(token)) {
    return failure('"token" must be 64 lowercase hexadecimal characters');
  }
  if (typeof version !== 'string' || version.length === 0) {
    return failure('"version" must be a non-empty string');
  }
  return { ok: true, handshake: { port, token, version } };
}

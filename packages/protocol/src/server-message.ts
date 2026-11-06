import type { ServerMessage } from './generated';

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

function isResponse(message: Record<string, unknown>): boolean {
  const id = message['id'];
  const hasOutcome = isRecord(message['result']) || isRecord(message['error']);
  return (id === null || typeof id === 'number') && hasOutcome;
}

function isEvent(message: Record<string, unknown>): boolean {
  const event = message['event'];
  return isRecord(event) && typeof event['kind'] === 'string';
}

/**
 * Parses a text frame from se-core. Checks the envelope only: payloads are trusted because both
 * sides are generated from the same Rust types. Returns `null` for anything else.
 */
export function parseServerMessage(frame: unknown): ServerMessage | null {
  if (typeof frame !== 'string') {
    return null;
  }
  let message: unknown;
  try {
    message = JSON.parse(frame);
  } catch {
    return null;
  }
  if (!isRecord(message)) {
    return null;
  }
  const valid =
    (message['type'] === 'response' && isResponse(message)) ||
    (message['type'] === 'event' && isEvent(message));
  // The checks above validate the envelope; the payload shape is guaranteed by codegen.
  return valid ? (message as ServerMessage) : null;
}

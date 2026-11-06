/**
 * Hand-written subset of the `/rpc` protocol needed for the liveness ping.
 * TODO(prompt-02): replace with the generated types and the `CoreClient` of `@se/protocol`.
 */
import type { CoreConnection } from '../../../electron/shared/bridge-api';

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

/** WebSocket URL of the core's RPC endpoint, carrying the session token. */
export function rpcSocketUrl(connection: CoreConnection): string {
  const url = new URL('/rpc', connection.url);
  url.protocol = 'ws:';
  url.searchParams.set('t', connection.token);
  return url.toString();
}

/** JSON text of a `ping` request. */
export function encodePingRequest(id: number): string {
  return JSON.stringify({ type: 'request', id, command: { kind: 'ping' } });
}

/** Version carried by the `pong` answering request `id`, or `null` for any other frame. */
export function parsePongVersion(frame: unknown, id: number): string | null {
  if (typeof frame !== 'string') {
    return null;
  }
  let message: unknown;
  try {
    message = JSON.parse(frame);
  } catch {
    return null;
  }
  if (!isRecord(message) || message['type'] !== 'response' || message['id'] !== id) {
    return null;
  }
  const result = message['result'];
  if (!isRecord(result) || result['kind'] !== 'pong' || typeof result['version'] !== 'string') {
    return null;
  }
  return result['version'];
}

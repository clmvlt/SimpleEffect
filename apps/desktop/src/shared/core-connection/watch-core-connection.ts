import type { SimpleEffectBridge } from '../../../electron/shared/bridge-api';
import type { CoreStatus } from './core-status';
import { encodePingRequest, parsePongVersion, rpcSocketUrl } from './rpc-messages';

/** Pause between a disconnect and the next attempt; the supervisor handles the core's backoff. */
const RECONNECT_DELAY_MS = 500;
const PING_REQUEST_ID = 1;

/**
 * Keeps a live WebSocket to se-core: asks the bridge for the connection, pings, and starts over
 * after any disconnect (a restarted core has a new port and token).
 *
 * Returns a function that stops watching and closes the socket.
 */
export function watchCoreConnection(
  bridge: SimpleEffectBridge,
  onStatus: (status: CoreStatus) => void,
): () => void {
  let stopped = false;
  let socket: WebSocket | null = null;
  let retryTimer: ReturnType<typeof setTimeout> | undefined;

  const retryLater = () => {
    if (!stopped) {
      onStatus({ kind: 'reconnecting' });
      retryTimer = setTimeout(() => void connect(), RECONNECT_DELAY_MS);
    }
  };

  const connect = async () => {
    try {
      const connection = await bridge.getCoreConnection();
      if (stopped) {
        return;
      }
      socket = new WebSocket(rpcSocketUrl(connection));
      socket.addEventListener('open', () => socket?.send(encodePingRequest(PING_REQUEST_ID)));
      socket.addEventListener('message', (event: MessageEvent<unknown>) => {
        const version = parsePongVersion(event.data, PING_REQUEST_ID);
        if (version !== null) {
          onStatus({ kind: 'connected', version });
        }
      });
      socket.addEventListener('close', retryLater);
    } catch {
      retryLater();
    }
  };

  onStatus({ kind: 'connecting' });
  void connect();

  return () => {
    stopped = true;
    clearTimeout(retryTimer);
    socket?.close();
  };
}

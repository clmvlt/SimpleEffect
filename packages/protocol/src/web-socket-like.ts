/**
 * The subset of the DOM `WebSocket` used by {@link CoreClient}, so tests can inject a fake.
 * A real `WebSocket` satisfies it.
 */
export interface WebSocketLike {
  send(data: string): void;
  close(): void;
  addEventListener(type: 'open' | 'close' | 'error', listener: () => void): void;
  addEventListener(type: 'message', listener: (event: { readonly data: unknown }) => void): void;
}

/** Opens a socket to `url`. */
export type WebSocketFactory = (url: string) => WebSocketLike;

/** Uses the platform `WebSocket` (renderer, browsers, Node.js ≥ 22). */
export const openWebSocket: WebSocketFactory = (url) => new WebSocket(url);

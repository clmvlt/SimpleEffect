import type { ClientMessage, ServerMessage } from '../generated';
import type { WebSocketFactory, WebSocketLike } from '../web-socket-like';

type Listener = (event: { readonly data: unknown }) => void;

/** In-memory {@link WebSocketLike} driven by tests playing the se-core side. */
export class FakeWebSocket implements WebSocketLike {
  readonly url: string;
  readonly sent: string[] = [];
  closedByClient = false;
  readonly #listeners = new Map<string, Set<Listener>>();

  constructor(url: string) {
    this.url = url;
  }

  send(data: string): void {
    this.sent.push(data);
  }

  close(): void {
    this.closedByClient = true;
  }

  addEventListener(type: string, listener: Listener): void {
    const listeners = this.#listeners.get(type) ?? new Set<Listener>();
    listeners.add(listener);
    this.#listeners.set(type, listeners);
  }

  /** The core accepted the connection. */
  acceptConnection(): void {
    this.#emit('open', undefined);
  }

  /** The core sent a frame (a message object is serialised; a string is sent as is). */
  receive(message: ServerMessage | string): void {
    this.#emit('message', typeof message === 'string' ? message : JSON.stringify(message));
  }

  /** The connection dropped (the core died or closed it). */
  drop(): void {
    this.#emit('close', undefined);
  }

  /** Requests written by the client, parsed. */
  requests(): ClientMessage[] {
    return this.sent.map((frame) => JSON.parse(frame) as ClientMessage);
  }

  #emit(type: string, data: unknown): void {
    for (const listener of this.#listeners.get(type) ?? []) {
      listener({ data });
    }
  }
}

/** A socket factory remembering every socket it opened, newest last. */
export function fakeSocketFactory(): { open: WebSocketFactory; sockets: FakeWebSocket[] } {
  const sockets: FakeWebSocket[] = [];
  return {
    sockets,
    open: (url) => {
      const socket = new FakeWebSocket(url);
      sockets.push(socket);
      return socket;
    },
  };
}

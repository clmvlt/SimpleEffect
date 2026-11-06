import { type CoreEndpointSource, resolveEndpoint, rpcSocketUrl } from './core-endpoint';
import { CoreRequestError } from './core-request-error';
import type {
  ClientMessage,
  Command,
  CoreEvent,
  ProtocolError,
  ResponseEnvelope,
} from './generated';
import { PendingRequests } from './pending-requests';
import type { CoreEventKind, CoreEventOf, ResultFor } from './protocol-types';
import {
  DEFAULT_RECONNECT_DELAY,
  type ReconnectDelayOptions,
  reconnectDelayMs,
} from './reconnect-delay';
import { parseServerMessage } from './server-message';
import { openWebSocket, type WebSocketFactory, type WebSocketLike } from './web-socket-like';

/**
 * - `connecting`: no connection has been established yet (first attempts).
 * - `open`: requests are sent immediately.
 * - `reconnecting`: the connection dropped; requests wait until it is back.
 * - `closed`: {@link CoreClient.close} was called; the client is unusable.
 */
export type ConnectionState = 'connecting' | 'open' | 'reconnecting' | 'closed';

/** Options of {@link CoreClient.connect}. */
export interface CoreClientOptions {
  /** Default request timeout. Defaults to 30 s. */
  readonly requestTimeoutMs?: number;
  /** Reconnection backoff. Defaults to 250 ms doubling up to 5 s. */
  readonly reconnectDelay?: ReconnectDelayOptions;
  /** Socket factory, replaced by a fake in tests. Defaults to the platform `WebSocket`. */
  readonly openSocket?: WebSocketFactory;
  /** Receives protocol anomalies (unreadable frames, failing handlers). Defaults to `console`. */
  readonly logger?: Pick<Console, 'warn'>;
}

/** Per-request options of {@link CoreClient.send}. */
export interface SendOptions {
  /** Overrides the client's default timeout for this request. */
  readonly timeoutMs?: number;
}

const DEFAULT_REQUEST_TIMEOUT_MS = 30_000;

const CONNECTION_LOST: ProtocolError = {
  code: 'CONNECTION_LOST',
  message:
    'The connection to se-core dropped before the answer arrived: the command may or may not have run. The client reconnects by itself; check the current state before retrying.',
};

const CLIENT_CLOSED: ProtocolError = {
  code: 'CLIENT_CLOSED',
  message: 'This CoreClient was closed: create a new one to talk to se-core.',
};

type EventHandler = (event: CoreEvent) => void;

/**
 * Typed client of se-core's `/rpc` WebSocket (ADR-0010, ADR-0012).
 *
 * Correlates responses by id, times requests out, pushes events to subscribers and reconnects
 * with a capped exponential backoff. Requests sent while disconnected wait for the next
 * connection; requests already sent when the connection drops fail with `CONNECTION_LOST`.
 */
export class CoreClient {
  readonly #source: CoreEndpointSource;
  readonly #requestTimeoutMs: number;
  readonly #reconnectDelay: ReconnectDelayOptions;
  readonly #openSocket: WebSocketFactory;
  readonly #logger: Pick<Console, 'warn'>;
  readonly #pending = new PendingRequests();
  readonly #eventHandlers = new Map<CoreEventKind, Set<EventHandler>>();
  readonly #stateListeners = new Set<(state: ConnectionState) => void>();
  #state: ConnectionState = 'connecting';
  #socket: WebSocketLike | null = null;
  #nextRequestId = 1;
  #consecutiveFailures = 0;
  #hasBeenOpen = false;
  #retryTimer: ReturnType<typeof setTimeout> | undefined;

  private constructor(source: CoreEndpointSource, options: CoreClientOptions) {
    this.#source = source;
    this.#requestTimeoutMs = options.requestTimeoutMs ?? DEFAULT_REQUEST_TIMEOUT_MS;
    this.#reconnectDelay = options.reconnectDelay ?? DEFAULT_RECONNECT_DELAY;
    this.#openSocket = options.openSocket ?? openWebSocket;
    this.#logger = options.logger ?? console;
  }

  /**
   * Creates a client and starts connecting to `source`. Pass a function when the core can
   * restart with a new port and token: it is called again before every reconnection attempt.
   */
  static connect(source: CoreEndpointSource, options: CoreClientOptions = {}): CoreClient {
    const client = new CoreClient(source, options);
    client.#attempt();
    return client;
  }

  /** Current connection state. */
  get state(): ConnectionState {
    return this.#state;
  }

  /** Calls `listener` on every state change; returns the unsubscribe function. */
  onStateChange(listener: (state: ConnectionState) => void): () => void {
    this.#stateListeners.add(listener);
    return () => {
      this.#stateListeners.delete(listener);
    };
  }

  /**
   * Sends `command` and resolves with its result, typed from the command `kind`.
   * Rejects with a {@link CoreRequestError} (see its `code`).
   */
  send<C extends Command>(command: C, options: SendOptions = {}): Promise<ResultFor<C['kind']>> {
    if (this.#state === 'closed') {
      return Promise.reject(new CoreRequestError(command.kind, CLIENT_CLOSED));
    }
    const id = this.#nextRequestId++;
    const message: ClientMessage = { type: 'request', id, command };
    const timeoutMs = options.timeoutMs ?? this.#requestTimeoutMs;
    const result = this.#pending.add(id, command.kind, JSON.stringify(message), timeoutMs);
    this.#flush();
    // `PendingRequests.settle` only resolves with a result whose kind matches the command's.
    return result as Promise<ResultFor<C['kind']>>;
  }

  /** Calls `handler` for every event of `kind`; returns the unsubscribe function. */
  on<K extends CoreEventKind>(kind: K, handler: (event: CoreEventOf<K>) => void): () => void {
    // Handlers are stored per kind and only ever called with events of that kind.
    const stored: EventHandler = (event) => {
      handler(event as CoreEventOf<K>);
    };
    const handlers = this.#eventHandlers.get(kind) ?? new Set<EventHandler>();
    handlers.add(stored);
    this.#eventHandlers.set(kind, handlers);
    return () => {
      handlers.delete(stored);
    };
  }

  /** Closes the connection for good and fails every pending request with `CLIENT_CLOSED`. */
  close(): void {
    if (this.#state === 'closed') {
      return;
    }
    this.#setState('closed');
    clearTimeout(this.#retryTimer);
    const socket = this.#socket;
    this.#socket = null;
    socket?.close();
    this.#pending.failAll(CLIENT_CLOSED);
  }

  #attempt(): void {
    resolveEndpoint(this.#source).then(
      (endpoint) => {
        this.#open(rpcSocketUrl(endpoint));
      },
      (error: unknown) => {
        this.#logger.warn(`[CoreClient] cannot resolve the se-core endpoint: ${String(error)}`);
        this.#retryLater();
      },
    );
  }

  #open(url: string): void {
    if (this.#state === 'closed') {
      return;
    }
    let socket: WebSocketLike;
    try {
      socket = this.#openSocket(url);
    } catch (error) {
      this.#logger.warn(`[CoreClient] cannot open the se-core socket: ${String(error)}`);
      this.#retryLater();
      return;
    }
    this.#socket = socket;
    // Events of a replaced socket are ignored.
    socket.addEventListener('open', () => {
      if (socket === this.#socket) this.#handleOpen();
    });
    socket.addEventListener('message', (event) => {
      if (socket === this.#socket) this.#handleFrame(event.data);
    });
    socket.addEventListener('close', () => {
      if (socket === this.#socket) this.#handleClose();
    });
  }

  #handleOpen(): void {
    this.#consecutiveFailures = 0;
    this.#hasBeenOpen = true;
    this.#setState('open');
    this.#flush();
  }

  #handleClose(): void {
    this.#socket = null;
    this.#pending.failSent(CONNECTION_LOST);
    this.#retryLater();
  }

  #retryLater(): void {
    if (this.#state === 'closed') {
      return;
    }
    this.#setState(this.#hasBeenOpen ? 'reconnecting' : 'connecting');
    const delay = reconnectDelayMs(this.#consecutiveFailures, this.#reconnectDelay);
    this.#consecutiveFailures += 1;
    this.#retryTimer = setTimeout(() => {
      this.#attempt();
    }, delay);
  }

  /** Writes every request not sent yet, if the connection is open. */
  #flush(): void {
    const socket = this.#socket;
    if (this.#state !== 'open' || socket === null) {
      return;
    }
    for (const frame of this.#pending.takeUnsent()) {
      socket.send(frame);
    }
  }

  #handleFrame(frame: unknown): void {
    const message = parseServerMessage(frame);
    if (message === null) {
      this.#logger.warn('[CoreClient] ignored a frame that is not a se-core message');
    } else if (message.type === 'event') {
      this.#dispatchEvent(message.event);
    } else {
      this.#handleResponse(message);
    }
  }

  #handleResponse(response: ResponseEnvelope): void {
    if (response.id !== null) {
      this.#pending.settle(response.id, response);
      return;
    }
    const reason = 'error' in response ? response.error.message : 'no details';
    this.#logger.warn(`[CoreClient] se-core could not read a request: ${reason}`);
  }

  #dispatchEvent(event: CoreEvent): void {
    for (const handler of this.#eventHandlers.get(event.kind) ?? []) {
      try {
        handler(event);
      } catch (error) {
        this.#logger.warn(`[CoreClient] a "${event.kind}" handler threw: ${String(error)}`);
      }
    }
  }

  #setState(state: ConnectionState): void {
    if (state === this.#state) {
      return;
    }
    this.#state = state;
    for (const listener of this.#stateListeners) {
      listener(state);
    }
  }
}

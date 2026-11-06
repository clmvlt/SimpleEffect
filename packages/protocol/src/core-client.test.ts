import { afterEach, beforeEach, describe, expect, expectTypeOf, it, vi } from 'vitest';

import { CoreClient, type ConnectionState } from './core-client';
import type { CoreEndpoint } from './core-endpoint';
import { CoreRequestError } from './core-request-error';
import type { CommandResult, CoreInfo, ServerMessage } from './generated';
import type { ResultFor } from './protocol-types';
import { type FakeWebSocket, fakeSocketFactory } from './test-support/fake-web-socket';

const ENDPOINT: CoreEndpoint = { url: 'http://127.0.0.1:4000', token: 'secret' };

const INFO: CoreInfo = {
  version: '0.1.0',
  os: 'windows',
  arch: 'x86_64',
  appDataDir: 'C:\\data',
  logsDir: 'C:\\data\\logs',
};

function response(id: number, result: CommandResult): ServerMessage {
  return { type: 'response', id, result };
}

/** Lets resolved promises (endpoint resolution, settled requests) run. */
async function flush(): Promise<void> {
  await vi.advanceTimersByTimeAsync(0);
}

function lastSocket(sockets: FakeWebSocket[]): FakeWebSocket {
  const socket = sockets.at(-1);
  if (socket === undefined) {
    throw new Error('no socket was opened');
  }
  return socket;
}

/** A client whose first socket is already open. */
async function openClient(options: { requestTimeoutMs?: number } = {}) {
  const factory = fakeSocketFactory();
  const logger = { warn: vi.fn() };
  const client = CoreClient.connect(ENDPOINT, { openSocket: factory.open, logger, ...options });
  await flush();
  lastSocket(factory.sockets).acceptConnection();
  return { client, sockets: factory.sockets, logger, socket: () => lastSocket(factory.sockets) };
}

beforeEach(() => {
  vi.useFakeTimers();
});

afterEach(() => {
  vi.useRealTimers();
});

describe('CoreClient requests', () => {
  it('connects to /rpc with the token and reports the open state', async () => {
    const { client, socket } = await openClient();
    expect(socket().url).toBe('ws://127.0.0.1:4000/rpc?t=secret');
    expect(client.state).toBe('open');
  });

  it('correlates out-of-order responses with their requests by id', async () => {
    const { client, socket } = await openClient();
    const ping = client.send({ kind: 'ping' });
    const info = client.send({ kind: 'getCoreInfo' });
    const [first, second] = socket().requests();
    expect(first).toEqual({ type: 'request', id: 1, command: { kind: 'ping' } });
    expect(second).toEqual({ type: 'request', id: 2, command: { kind: 'getCoreInfo' } });

    socket().receive(response(2, { kind: 'getCoreInfo', ...INFO }));
    socket().receive(response(1, { kind: 'ping' }));

    await expect(info).resolves.toEqual({ kind: 'getCoreInfo', ...INFO });
    await expect(ping).resolves.toEqual({ kind: 'ping' });
  });

  it('types each result from the command kind', async () => {
    const { client } = await openClient();
    const info = client.send({ kind: 'getCoreInfo' });
    expectTypeOf(info).resolves.toEqualTypeOf<ResultFor<'getCoreInfo'>>();
    expectTypeOf(info).resolves.toExtend<CoreInfo>();
    client.close();
    await expect(info).rejects.toThrow(CoreRequestError);
  });

  it('queues requests until the socket opens', async () => {
    const factory = fakeSocketFactory();
    const client = CoreClient.connect(ENDPOINT, { openSocket: factory.open });
    const ping = client.send({ kind: 'ping' });
    await flush();
    const socket = lastSocket(factory.sockets);
    expect(socket.sent).toEqual([]);

    socket.acceptConnection();
    expect(socket.requests()).toEqual([{ type: 'request', id: 1, command: { kind: 'ping' } }]);
    socket.receive(response(1, { kind: 'ping' }));
    await expect(ping).resolves.toEqual({ kind: 'ping' });
  });

  it('rejects with the error code and details sent by se-core', async () => {
    const { client, socket } = await openClient();
    const ping = client.send({ kind: 'ping' });
    socket().receive({
      type: 'response',
      id: 1,
      error: { code: 'UNKNOWN_COMMAND', message: 'nope', details: { kind: 'ping' } },
    });
    await expect(ping).rejects.toMatchObject({
      code: 'UNKNOWN_COMMAND',
      command: 'ping',
      details: { kind: 'ping' },
    });
  });

  it('rejects a result whose kind does not match the command as INTERNAL', async () => {
    const { client, socket } = await openClient();
    const ping = client.send({ kind: 'ping' });
    socket().receive(response(1, { kind: 'getCoreInfo', ...INFO }));
    await expect(ping).rejects.toMatchObject({ code: 'INTERNAL' });
  });

  it('times requests out after 30 s by default, or after the given timeout', async () => {
    const { client, socket } = await openClient();
    const slow = client.send({ kind: 'ping' });
    const quick = client.send({ kind: 'ping' }, { timeoutMs: 100 });
    const outcomes = Promise.allSettled([slow, quick]);

    await vi.advanceTimersByTimeAsync(100);
    await expect(quick).rejects.toMatchObject({ code: 'TIMEOUT' });
    await vi.advanceTimersByTimeAsync(29_900);
    await expect(slow).rejects.toMatchObject({ code: 'TIMEOUT' });
    await outcomes;

    // A late response is ignored.
    socket().receive(response(1, { kind: 'ping' }));
  });

  it('ignores unreadable frames and warns', async () => {
    const { socket, logger } = await openClient();
    socket().receive('not json');
    socket().receive('{"type":"mystery"}');
    expect(logger.warn).toHaveBeenCalledTimes(2);
  });
});

describe('CoreClient reconnection', () => {
  it('fails in-flight requests with CONNECTION_LOST and keeps unsent ones for the next socket', async () => {
    const { client, socket, sockets } = await openClient();
    const inFlight = client.send({ kind: 'ping' });
    socket().drop();
    await expect(inFlight).rejects.toMatchObject({ code: 'CONNECTION_LOST' });
    expect(client.state).toBe('reconnecting');

    const queued = client.send({ kind: 'ping' });
    await vi.advanceTimersByTimeAsync(250);
    expect(sockets).toHaveLength(2);
    socket().acceptConnection();
    expect(client.state).toBe('open');
    expect(socket().requests()).toEqual([{ type: 'request', id: 2, command: { kind: 'ping' } }]);
    socket().receive(response(2, { kind: 'ping' }));
    await expect(queued).resolves.toEqual({ kind: 'ping' });
  });

  it('retries with an exponential backoff and resolves the endpoint again each time', async () => {
    const factory = fakeSocketFactory();
    let port = 4000;
    const resolveEndpoint = vi.fn(() => {
      port += 1;
      return Promise.resolve({ url: `http://127.0.0.1:${String(port)}`, token: 't' });
    });
    const client = CoreClient.connect(resolveEndpoint, { openSocket: factory.open });
    await flush();
    lastSocket(factory.sockets).drop();
    expect(client.state).toBe('connecting');

    await vi.advanceTimersByTimeAsync(249);
    expect(factory.sockets).toHaveLength(1);
    await vi.advanceTimersByTimeAsync(1);
    expect(factory.sockets).toHaveLength(2);

    lastSocket(factory.sockets).drop();
    await vi.advanceTimersByTimeAsync(499);
    expect(factory.sockets).toHaveLength(2);
    await vi.advanceTimersByTimeAsync(1);
    expect(factory.sockets).toHaveLength(3);
    expect(lastSocket(factory.sockets).url).toBe('ws://127.0.0.1:4003/rpc?t=t');
    expect(resolveEndpoint).toHaveBeenCalledTimes(3);
    client.close();
  });

  it('retries when the endpoint cannot be resolved', async () => {
    const factory = fakeSocketFactory();
    const resolveEndpoint = vi
      .fn<() => Promise<CoreEndpoint>>()
      .mockRejectedValueOnce(new Error('core not started'))
      .mockResolvedValue(ENDPOINT);
    const client = CoreClient.connect(resolveEndpoint, {
      openSocket: factory.open,
      logger: { warn: vi.fn() },
    });
    await flush();
    expect(factory.sockets).toHaveLength(0);
    await vi.advanceTimersByTimeAsync(250);
    expect(factory.sockets).toHaveLength(1);
    client.close();
  });

  it('notifies state listeners on every change', async () => {
    const factory = fakeSocketFactory();
    const client = CoreClient.connect(ENDPOINT, { openSocket: factory.open });
    const states: ConnectionState[] = [];
    client.onStateChange((state) => states.push(state));
    await flush();
    lastSocket(factory.sockets).acceptConnection();
    lastSocket(factory.sockets).drop();
    await vi.advanceTimersByTimeAsync(250);
    lastSocket(factory.sockets).acceptConnection();
    client.close();
    expect(states).toEqual(['open', 'reconnecting', 'open', 'closed']);
  });
});

describe('CoreClient events and closing', () => {
  it('delivers events to the handlers of their kind until they unsubscribe', async () => {
    const { client, socket } = await openClient();
    const onResync = vi.fn();
    const onShutdown = vi.fn();
    const unsubscribe = client.on('resyncRequired', onResync);
    client.on('coreShuttingDown', onShutdown);

    socket().receive({ type: 'event', event: { kind: 'resyncRequired', missedEvents: 4 } });
    expect(onResync).toHaveBeenCalledWith({ kind: 'resyncRequired', missedEvents: 4 });
    expect(onShutdown).not.toHaveBeenCalled();

    unsubscribe();
    socket().receive({ type: 'event', event: { kind: 'resyncRequired', missedEvents: 1 } });
    expect(onResync).toHaveBeenCalledTimes(1);
  });

  it('keeps calling other handlers when one throws', async () => {
    const { client, socket, logger } = await openClient();
    const healthy = vi.fn();
    client.on('coreShuttingDown', () => {
      throw new Error('boom');
    });
    client.on('coreShuttingDown', healthy);
    socket().receive({ type: 'event', event: { kind: 'coreShuttingDown' } });
    expect(healthy).toHaveBeenCalledTimes(1);
    expect(logger.warn).toHaveBeenCalledOnce();
  });

  it('close() fails pending requests, closes the socket and never reconnects', async () => {
    const { client, socket, sockets } = await openClient();
    const pending = client.send({ kind: 'ping' });
    client.close();
    await expect(pending).rejects.toMatchObject({ code: 'CLIENT_CLOSED' });
    expect(socket().closedByClient).toBe(true);
    expect(client.state).toBe('closed');

    socket().drop();
    await vi.advanceTimersByTimeAsync(10_000);
    expect(sockets).toHaveLength(1);
    await expect(client.send({ kind: 'ping' })).rejects.toMatchObject({ code: 'CLIENT_CLOSED' });
  });
});

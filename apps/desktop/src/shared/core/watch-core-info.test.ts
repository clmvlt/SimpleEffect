import type { ConnectionState, CoreInfo, ResultFor } from '@se/protocol';
import { describe, expect, it, vi } from 'vitest';

import { type CoreInfoSource, watchCoreInfo } from './watch-core-info';

const INFO: CoreInfo = {
  version: '0.1.0',
  os: 'linux',
  arch: 'x86_64',
  appDataDir: '/data',
  logsDir: '/data/logs',
};

/** A controllable stand-in for `CoreClient`: tests change the state and answer requests. */
function fakeClient(initialState: ConnectionState) {
  const listeners = new Set<(state: ConnectionState) => void>();
  const answers: ((result: ResultFor<'getCoreInfo'>) => void)[] = [];
  const client = {
    state: initialState,
    onStateChange: (listener: (state: ConnectionState) => void) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    send: vi.fn(
      () =>
        new Promise<ResultFor<'getCoreInfo'>>((resolve) => {
          answers.push(resolve);
        }),
    ),
  };
  return {
    // The fake only implements `getCoreInfo`, the one command `watchCoreInfo` sends.
    client: client as unknown as CoreInfoSource,
    setState(state: ConnectionState) {
      client.state = state;
      for (const listener of listeners) listener(state);
    },
    answer(index: number, info: CoreInfo = INFO) {
      answers[index]?.({ kind: 'getCoreInfo', ...info });
    },
    sendCount: () => client.send.mock.calls.length,
  };
}

describe('watchCoreInfo', () => {
  it('fetches the info once the connection opens', async () => {
    const fake = fakeClient('connecting');
    const onInfo = vi.fn();
    watchCoreInfo(fake.client, onInfo);
    expect(fake.sendCount()).toBe(0);

    fake.setState('open');
    fake.answer(0);
    await vi.waitFor(() => {
      expect(onInfo).toHaveBeenLastCalledWith(expect.objectContaining(INFO));
    });
  });

  it('fetches immediately when already open and again after a reconnection', async () => {
    const fake = fakeClient('open');
    const onInfo = vi.fn();
    watchCoreInfo(fake.client, onInfo);
    expect(fake.sendCount()).toBe(1);

    fake.setState('reconnecting');
    expect(onInfo).toHaveBeenLastCalledWith(null);
    fake.setState('open');
    expect(fake.sendCount()).toBe(2);
    fake.answer(1, { ...INFO, version: '0.2.0' });
    await vi.waitFor(() => {
      expect(onInfo).toHaveBeenLastCalledWith(expect.objectContaining({ version: '0.2.0' }));
    });
  });

  it('drops answers that arrive after the connection changed or after stopping', async () => {
    const fake = fakeClient('open');
    const onInfo = vi.fn();
    const stop = watchCoreInfo(fake.client, onInfo);
    fake.setState('reconnecting');
    fake.answer(0);
    fake.setState('open');
    stop();
    fake.answer(1);
    await Promise.resolve();
    await Promise.resolve();
    expect(onInfo).not.toHaveBeenCalledWith(expect.objectContaining(INFO));
  });
});

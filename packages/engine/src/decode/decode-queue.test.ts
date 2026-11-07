import { describe, expect, it } from 'vitest';

import { DecodeQueue } from './decode-queue';

function deferred<T>() {
  let resolve: (value: T) => void = () => undefined;
  const promise = new Promise<T>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

describe('DecodeQueue', () => {
  it('runs tasks one at a time, in order', async () => {
    const queue = new DecodeQueue();
    const started: number[] = [];
    const gate = deferred<undefined>();
    const first = queue.enqueue(async () => {
      started.push(1);
      await gate.promise;
      return 1;
    });
    const second = queue.enqueue(() => {
      started.push(2);
      return Promise.resolve(2);
    });
    await Promise.resolve();
    expect(started).toEqual([1]);
    expect(queue.size).toBe(2);
    gate.resolve(undefined);
    expect(await first).toBe(1);
    expect(await second).toBe(2);
    expect(started).toEqual([1, 2]);
  });

  it('skips tasks aborted before they start', async () => {
    const queue = new DecodeQueue();
    const gate = deferred<undefined>();
    const controller = new AbortController();
    let ran = false;
    const running = queue.enqueue(async () => {
      await gate.promise;
      return 'kept';
    }, controller.signal);
    const waiting = queue.enqueue(() => {
      ran = true;
      return Promise.resolve('skipped');
    }, controller.signal);
    controller.abort();
    expect(await waiting).toBeNull();
    gate.resolve(undefined);
    expect(await running).toBe('kept');
    expect(ran).toBe(false);
    expect(await queue.enqueue(() => Promise.resolve(1), controller.signal)).toBeNull();
  });

  it('rejects with the task error and carries on', async () => {
    const queue = new DecodeQueue();
    const failing = queue.enqueue(() => Promise.reject(new Error('decoder error')));
    const next = queue.enqueue(() => Promise.resolve('ok'));
    await expect(failing).rejects.toThrow('decoder error');
    expect(await next).toBe('ok');
  });
});

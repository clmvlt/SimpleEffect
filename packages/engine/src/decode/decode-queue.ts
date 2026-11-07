interface QueuedTask {
  readonly run: () => Promise<void>;
  readonly signal: AbortSignal | undefined;
  readonly skip: () => void;
}

/**
 * Runs decode tasks of one source one at a time, in order. A task whose signal is aborted before
 * it starts is skipped (its promise resolves `null`): during a fast scrub, only the frames still
 * wanted get decoded. A task already running finishes (WebCodecs cannot stop mid-GOP) and its
 * result still reaches the cache.
 */
export class DecodeQueue {
  private readonly pending: QueuedTask[] = [];
  private running = false;

  /** Tasks waiting or running. */
  get size(): number {
    return this.pending.length + (this.running ? 1 : 0);
  }

  /** Queues `task`; resolves with its result, or `null` if `signal` aborted before it started. */
  enqueue<T>(task: () => Promise<T>, signal?: AbortSignal): Promise<T | null> {
    if (signal?.aborted === true) return Promise.resolve(null);
    return new Promise<T | null>((resolve, reject) => {
      const queued: QueuedTask = {
        signal,
        skip: () => {
          resolve(null);
        },
        run: async () => {
          try {
            resolve(await task());
          } catch (error: unknown) {
            reject(error instanceof Error ? error : new Error(String(error)));
          }
        },
      };
      this.pending.push(queued);
      signal?.addEventListener(
        'abort',
        () => {
          this.cancel(queued);
        },
        { once: true },
      );
      void this.drain();
    });
  }

  private cancel(task: QueuedTask): void {
    const index = this.pending.indexOf(task);
    if (index < 0) return;
    this.pending.splice(index, 1);
    task.skip();
  }

  private async drain(): Promise<void> {
    if (this.running) return;
    this.running = true;
    try {
      for (let task = this.pending.shift(); task !== undefined; task = this.pending.shift()) {
        if (task.signal?.aborted === true) task.skip();
        else await task.run();
      }
    } finally {
      this.running = false;
    }
  }
}

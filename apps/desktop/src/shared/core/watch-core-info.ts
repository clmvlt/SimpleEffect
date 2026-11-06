import type { CoreClient, CoreInfo } from '@se/protocol';

/** The parts of {@link CoreClient} needed here (a fake implements them in tests). */
export type CoreInfoSource = Pick<CoreClient, 'state' | 'onStateChange' | 'send'>;

/**
 * Fetches `getCoreInfo` every time the connection opens (a reconnection may reach a restarted
 * core) and reports `null` whenever it is not open. Answers arriving after the connection
 * changed again are dropped. Returns a function that stops watching.
 */
export function watchCoreInfo(
  client: CoreInfoSource,
  onInfo: (info: CoreInfo | null) => void,
): () => void {
  let generation = 0;
  let stopped = false;

  const refresh = () => {
    generation += 1;
    const current = generation;
    if (client.state !== 'open') {
      onInfo(null);
      return;
    }
    client.send({ kind: 'getCoreInfo' }).then(
      // The `getCoreInfo` result is a `CoreInfo` tagged with its `kind`.
      (info) => {
        if (!stopped && current === generation) {
          onInfo(info);
        }
      },
      // A failed request leaves the info empty; the next connection retries it.
      () => undefined,
    );
  };

  const unsubscribe = client.onStateChange(refresh);
  if (client.state === 'open') {
    refresh();
  }
  return () => {
    stopped = true;
    unsubscribe();
  };
}

import type { AiStatus, CoreClient } from '@se/protocol';

/** The parts of {@link CoreClient} needed here (a fake implements them in tests). */
export type AiStatusSource = Pick<CoreClient, 'state' | 'onStateChange' | 'send' | 'on'>;

/**
 * Keeps `onStatus` told about the AI provider: asks `getAiStatus` every time the connection opens
 * (a reconnection may reach a restarted core), follows `aiStatusChanged`, and reports `null`
 * while disconnected. Answers arriving after the connection changed again are dropped. Returns a
 * function that stops watching.
 */
export function watchAiStatus(
  client: AiStatusSource,
  onStatus: (status: AiStatus | null) => void,
): () => void {
  let generation = 0;
  let stopped = false;
  const refresh = () => {
    generation += 1;
    const current = generation;
    if (client.state !== 'open') {
      onStatus(null);
      return;
    }
    client.send({ kind: 'getAiStatus' }).then(
      ({ status }) => {
        if (!stopped && current === generation) onStatus(status);
      },
      (error: unknown) => {
        console.warn('[ai] reading the AI status failed', error);
      },
    );
  };
  const unsubscribeState = client.onStateChange(refresh);
  const unsubscribeEvents = client.on('aiStatusChanged', (event) => {
    if (stopped) return;
    onStatus({
      provider: event.provider,
      installation: event.installation,
      auth: event.auth,
      capabilities: event.capabilities,
    });
  });
  if (client.state === 'open') refresh();
  return () => {
    stopped = true;
    unsubscribeState();
    unsubscribeEvents();
  };
}

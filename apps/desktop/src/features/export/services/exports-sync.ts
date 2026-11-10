import type { CoreClient, ExportInfo } from '@se/protocol';

import { exportsApi } from '../api';
import {
  dropFinishedExports,
  exportsStore,
  setExports,
  upsertExport,
} from '../store/exports-store';

/** The parts of {@link CoreClient} the sync needs (a fake implements them in tests). */
export type ExportsSyncSource = Pick<CoreClient, 'state' | 'onStateChange' | 'send' | 'on'>;

/** Called with an export's previous and new state on every update. */
export type ExportChangeListener = (previous: ExportInfo | undefined, next: ExportInfo) => void;

/**
 * Mirrors the core's exports into {@link exportsStore}: `listExports` each time the connection
 * opens (a restarted core has none), then `exportUpdated` and `exportsCleared`. Answers arriving
 * after the connection changed again are dropped. Returns the function that stops.
 */
export function connectExportsSync(
  client: ExportsSyncSource,
  onChange: ExportChangeListener,
): () => void {
  let generation = 0;
  let stopped = false;
  const load = () => {
    generation += 1;
    const current = generation;
    if (client.state !== 'open') return;
    exportsApi(client)
      .list()
      .then(
        (exports) => {
          if (!stopped && current === generation) setExports(exports);
        },
        (error: unknown) => {
          console.warn('[export] listing exports failed', error);
        },
      );
  };
  const unsubscribeState = client.onStateChange(load);
  const unsubscribeUpdates = client.on('exportUpdated', ({ export: info }) => {
    if (stopped) return;
    const previous = exportsStore.getState().exports.find((known) => known.id === info.id);
    upsertExport(info);
    onChange(previous, info);
  });
  const unsubscribeCleared = client.on('exportsCleared', () => {
    if (!stopped) dropFinishedExports();
  });
  if (client.state === 'open') load();
  return () => {
    stopped = true;
    unsubscribeState();
    unsubscribeUpdates();
    unsubscribeCleared();
  };
}

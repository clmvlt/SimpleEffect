import type { CoreEventKind, CoreEventOf } from '@se/protocol';
import { useEffect, useEffectEvent } from 'react';

import { useCoreClient } from './use-core-client';

/**
 * Calls `handler` for every core event of `kind` while the component is mounted. The latest
 * `handler` is always used, without resubscribing when it changes.
 */
export function useCoreEvent<K extends CoreEventKind>(
  kind: K,
  handler: (event: CoreEventOf<K>) => void,
): void {
  const client = useCoreClient();
  const onEvent = useEffectEvent(handler);
  useEffect(
    () =>
      client.on(kind, (event) => {
        onEvent(event);
      }),
    [client, kind],
  );
}

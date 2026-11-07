import type { CoreEndpoint } from '@se/protocol';
import { useSyncExternalStore } from 'react';

import { useCoreClient } from './use-core-client';

/**
 * The endpoint (origin and token) of the current core, for `/media` URLs; `null` before the first
 * connection attempt. Re-renders on connection changes: a restarted core has a new port and token.
 */
export function useCoreEndpoint(): CoreEndpoint | null {
  const client = useCoreClient();
  return useSyncExternalStore(
    (onChange) => client.onStateChange(onChange),
    () => client.endpoint,
  );
}

import type { ConnectionState } from '@se/protocol';
import { useSyncExternalStore } from 'react';

import { useCoreClient } from './use-core-client';

/** Live state of the connection to se-core (`connecting | open | reconnecting | closed`). */
export function useConnectionStatus(): ConnectionState {
  const client = useCoreClient();
  return useSyncExternalStore(
    (onChange) => client.onStateChange(onChange),
    () => client.state,
  );
}

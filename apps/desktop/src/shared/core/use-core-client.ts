import type { CoreClient } from '@se/protocol';
import { use } from 'react';

import { CoreClientContext } from './core-client-context';

/** The app's {@link CoreClient}. Throws outside `CoreClientProvider` (a wiring bug). */
export function useCoreClient(): CoreClient {
  const client = use(CoreClientContext);
  if (client === null) {
    throw new Error('useCoreClient() must be called below <CoreClientProvider> (see main.tsx).');
  }
  return client;
}

import type { CoreClient } from '@se/protocol';
import type { ReactNode } from 'react';

import { CoreClientContext } from './core-client-context';

/** Props of {@link CoreClientProvider}. */
export interface CoreClientProviderProps {
  readonly client: CoreClient;
  readonly children: ReactNode;
}

/** Makes `client` available to `useCoreClient()` and the other core hooks below it. */
export function CoreClientProvider({ client, children }: CoreClientProviderProps) {
  return <CoreClientContext value={client}>{children}</CoreClientContext>;
}

import type { CoreInfo } from '@se/protocol';
import { useEffect, useState } from 'react';

import { useCoreClient } from './use-core-client';
import { watchCoreInfo } from './watch-core-info';

/** Facts about the connected core (`getCoreInfo`), or `null` while disconnected or loading. */
export function useCoreInfo(): CoreInfo | null {
  const client = useCoreClient();
  const [info, setInfo] = useState<CoreInfo | null>(null);
  useEffect(() => watchCoreInfo(client, setInfo), [client]);
  return info;
}

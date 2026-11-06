import { useEffect, useState } from 'react';

import type { CoreStatus } from './core-status';
import { watchCoreConnection } from './watch-core-connection';

/** Live state of the connection to se-core, reconnecting automatically. */
export function useCoreStatus(): CoreStatus {
  const [status, setStatus] = useState<CoreStatus>({ kind: 'connecting' });
  useEffect(() => watchCoreConnection(window.simpleEffect, setStatus), []);
  return status;
}

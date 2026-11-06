import { CoreClient } from '@se/protocol';

import type { SimpleEffectBridge } from '../../../electron/shared/bridge-api';

/**
 * Creates the renderer's single {@link CoreClient}. The endpoint is asked from the Electron
 * bridge before every connection attempt: a restarted core has a new port and token, and the
 * bridge waits while the supervisor restarts it.
 */
export function createCoreClient(bridge: SimpleEffectBridge): CoreClient {
  return CoreClient.connect(() => bridge.getCoreConnection());
}

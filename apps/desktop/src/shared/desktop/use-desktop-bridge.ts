import { use } from 'react';

import type { SimpleEffectBridge } from '../../../electron/shared/bridge-api';
import { DesktopBridgeContext } from './desktop-bridge-context';

/** The Electron bridge: native dialogs, revealing files, the core endpoint. */
export function useDesktopBridge(): SimpleEffectBridge {
  const bridge = use(DesktopBridgeContext);
  if (bridge === null) {
    throw new Error('useDesktopBridge must be used inside <DesktopBridgeProvider>.');
  }
  return bridge;
}

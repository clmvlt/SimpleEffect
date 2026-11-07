import type { ReactNode } from 'react';

import type { SimpleEffectBridge } from '../../../electron/shared/bridge-api';
import { DesktopBridgeContext } from './desktop-bridge-context';

/** Props of {@link DesktopBridgeProvider}. */
export interface DesktopBridgeProviderProps {
  readonly bridge: SimpleEffectBridge;
  readonly children: ReactNode;
}

/** Provides the Electron bridge to `useDesktopBridge()`. */
export function DesktopBridgeProvider({ bridge, children }: DesktopBridgeProviderProps) {
  return <DesktopBridgeContext value={bridge}>{children}</DesktopBridgeContext>;
}

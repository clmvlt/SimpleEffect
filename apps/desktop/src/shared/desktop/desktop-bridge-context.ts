import { createContext } from 'react';

import type { SimpleEffectBridge } from '../../../electron/shared/bridge-api';

/** The Electron bridge (`window.simpleEffect`), injected so tests can pass a fake. */
export const DesktopBridgeContext = createContext<SimpleEffectBridge | null>(null);

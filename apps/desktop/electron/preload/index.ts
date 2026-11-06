/**
 * Preload script (sandboxed, CommonJS): exposes the minimal typed bridge `window.simpleEffect`.
 * It holds no logic; every call is forwarded to the main process over IPC.
 */
import { contextBridge, ipcRenderer } from 'electron';

import { IPC_CHANNELS, type CoreConnection, type SimpleEffectBridge } from '../shared/bridge-api';

const bridge: SimpleEffectBridge = {
  getCoreConnection: () =>
    ipcRenderer.invoke(IPC_CHANNELS.getCoreConnection) as Promise<CoreConnection>,
};

contextBridge.exposeInMainWorld('simpleEffect', bridge);

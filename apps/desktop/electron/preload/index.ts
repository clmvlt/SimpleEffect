/**
 * Preload script (sandboxed, CommonJS): exposes the minimal typed bridge `window.simpleEffect`.
 * It holds no logic; every call is forwarded to the main process over IPC.
 */
import { contextBridge, ipcRenderer } from 'electron';

import {
  IPC_CHANNELS,
  type ChooseFolderOptions,
  type CoreConnection,
  type SimpleEffectBridge,
} from '../shared/bridge-api';

const bridge: SimpleEffectBridge = {
  getCoreConnection: () =>
    ipcRenderer.invoke(IPC_CHANNELS.getCoreConnection) as Promise<CoreConnection>,
  chooseFolder: (options: ChooseFolderOptions) =>
    ipcRenderer.invoke(IPC_CHANNELS.chooseFolder, options) as Promise<string | null>,
  showInFolder: (path: string) =>
    ipcRenderer.invoke(IPC_CHANNELS.showInFolder, path) as Promise<void>,
};

contextBridge.exposeInMainWorld('simpleEffect', bridge);

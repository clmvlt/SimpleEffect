/**
 * Preload script (sandboxed, CommonJS): exposes the minimal typed bridge `window.simpleEffect`.
 * It holds no logic; every call is forwarded to the main process over IPC.
 */
import { contextBridge, ipcRenderer, webUtils } from 'electron';

import {
  IPC_CHANNELS,
  type ChooseFilesOptions,
  type ChooseFolderOptions,
  type CoreConnection,
  type SimpleEffectBridge,
} from '../shared/bridge-api';

const bridge: SimpleEffectBridge = {
  getCoreConnection: () =>
    ipcRenderer.invoke(IPC_CHANNELS.getCoreConnection) as Promise<CoreConnection>,
  chooseFolder: (options: ChooseFolderOptions) =>
    ipcRenderer.invoke(IPC_CHANNELS.chooseFolder, options) as Promise<string | null>,
  chooseFiles: (options: ChooseFilesOptions) =>
    ipcRenderer.invoke(IPC_CHANNELS.chooseFiles, options) as Promise<string[]>,
  showInFolder: (path: string) =>
    ipcRenderer.invoke(IPC_CHANNELS.showInFolder, path) as Promise<void>,
  getPathForFile: (file: File) => webUtils.getPathForFile(file),
};

contextBridge.exposeInMainWorld('simpleEffect', bridge);

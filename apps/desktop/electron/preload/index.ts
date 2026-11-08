/**
 * Preload script (sandboxed, CommonJS): exposes the minimal typed bridge `window.simpleEffect`.
 * It holds no logic; every call is forwarded to the main process over IPC.
 */
import { contextBridge, ipcRenderer, webUtils } from 'electron';

import {
  IPC_CHANNELS,
  type ChooseFileOptions,
  type ChooseFilesOptions,
  type ChooseSaveFileOptions,
  type ChooseFolderOptions,
  type CoreConnection,
  type SimpleEffectBridge,
  type SystemNotification,
} from '../shared/bridge-api';

const bridge: SimpleEffectBridge = {
  getCoreConnection: () =>
    ipcRenderer.invoke(IPC_CHANNELS.getCoreConnection) as Promise<CoreConnection>,
  chooseFolder: (options: ChooseFolderOptions) =>
    ipcRenderer.invoke(IPC_CHANNELS.chooseFolder, options) as Promise<string | null>,
  chooseFiles: (options: ChooseFilesOptions) =>
    ipcRenderer.invoke(IPC_CHANNELS.chooseFiles, options) as Promise<string[]>,
  chooseFile: (options: ChooseFileOptions) =>
    ipcRenderer.invoke(IPC_CHANNELS.chooseFile, options) as Promise<string | null>,
  chooseSaveFile: (options: ChooseSaveFileOptions) =>
    ipcRenderer.invoke(IPC_CHANNELS.chooseSaveFile, options) as Promise<string | null>,
  openExternal: (url: string) =>
    ipcRenderer.invoke(IPC_CHANNELS.openExternal, url) as Promise<void>,
  showInFolder: (path: string) =>
    ipcRenderer.invoke(IPC_CHANNELS.showInFolder, path) as Promise<void>,
  getPathForFile: (file: File) => webUtils.getPathForFile(file),
  showNotification: (notification: SystemNotification) =>
    ipcRenderer.invoke(IPC_CHANNELS.showNotification, notification) as Promise<void>,
  onNotificationClick: (listener: (tag: string) => void) => {
    const handler = (_event: Electron.IpcRendererEvent, tag: unknown) => {
      if (typeof tag === 'string') listener(tag);
    };
    ipcRenderer.on(IPC_CHANNELS.notificationClicked, handler);
    return () => {
      ipcRenderer.removeListener(IPC_CHANNELS.notificationClicked, handler);
    };
  },
};

contextBridge.exposeInMainWorld('simpleEffect', bridge);

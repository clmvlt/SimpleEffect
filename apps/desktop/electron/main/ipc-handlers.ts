import { BrowserWindow, dialog, ipcMain, shell } from 'electron';

import { IPC_CHANNELS } from '../shared/bridge-api';
import type { CoreSupervisor } from './core-supervisor';
import {
  parseChooseFilesOptions,
  parseChooseFolderOptions,
  parseRevealPath,
} from './ipc-arguments';

/** Answers the preload bridge's requests (see `electron/shared/bridge-api.ts`). */
export function registerIpcHandlers(supervisor: CoreSupervisor): void {
  ipcMain.handle(IPC_CHANNELS.getCoreConnection, () => supervisor.waitForConnection());

  ipcMain.handle(IPC_CHANNELS.chooseFolder, async (event, value: unknown) => {
    const options = parseChooseFolderOptions(value);
    const dialogOptions: Electron.OpenDialogOptions = {
      title: options.title,
      buttonLabel: options.buttonLabel,
      properties: ['openDirectory', 'createDirectory'],
      ...(options.defaultPath === undefined ? {} : { defaultPath: options.defaultPath }),
    };
    const window = BrowserWindow.fromWebContents(event.sender);
    const result =
      window === null
        ? await dialog.showOpenDialog(dialogOptions)
        : await dialog.showOpenDialog(window, dialogOptions);
    return result.canceled ? null : (result.filePaths[0] ?? null);
  });

  ipcMain.handle(IPC_CHANNELS.chooseFiles, async (event, value: unknown) => {
    const options = parseChooseFilesOptions(value);
    const dialogOptions: Electron.OpenDialogOptions = {
      title: options.title,
      buttonLabel: options.buttonLabel,
      properties: ['openFile', 'multiSelections'],
      filters: [
        { name: options.filterName, extensions: [...options.extensions] },
        { name: options.allFilesName, extensions: ['*'] },
      ],
    };
    const window = BrowserWindow.fromWebContents(event.sender);
    const result =
      window === null
        ? await dialog.showOpenDialog(dialogOptions)
        : await dialog.showOpenDialog(window, dialogOptions);
    return result.canceled ? [] : result.filePaths;
  });

  ipcMain.handle(IPC_CHANNELS.showInFolder, (_event, value: unknown) => {
    shell.showItemInFolder(parseRevealPath(value));
  });
}

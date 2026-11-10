import { existsSync } from 'node:fs';
import { join } from 'node:path';

import { BrowserWindow, dialog, ipcMain, shell } from 'electron';

import { IPC_CHANNELS } from '../shared/bridge-api';
import type { CoreSupervisor } from './core-supervisor';
import type { ExportActivityGuard } from './export-activity';
import { parseExportActivity, parseResolveFilePath } from './export-arguments';
import {
  parseChooseFileOptions,
  parseChooseFilesOptions,
  parseChooseSaveFileOptions,
  parseChooseFolderOptions,
  parseExternalUrl,
  parseNotification,
  parseRevealPath,
} from './ipc-arguments';
import { showSystemNotification } from './notifications';

/** Answers the preload bridge's requests (see `electron/shared/bridge-api.ts`). */
export function registerIpcHandlers(
  supervisor: CoreSupervisor,
  exportGuard: ExportActivityGuard,
): void {
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

  ipcMain.handle(IPC_CHANNELS.chooseFile, async (event, value: unknown) => {
    const options = parseChooseFileOptions(value);
    const dialogOptions: Electron.OpenDialogOptions = {
      title: options.title,
      buttonLabel: options.buttonLabel,
      properties: ['openFile', 'treatPackageAsDirectory'],
    };
    const window = BrowserWindow.fromWebContents(event.sender);
    const result =
      window === null
        ? await dialog.showOpenDialog(dialogOptions)
        : await dialog.showOpenDialog(window, dialogOptions);
    return result.canceled ? null : (result.filePaths[0] ?? null);
  });

  ipcMain.handle(IPC_CHANNELS.chooseSaveFile, async (event, value: unknown) => {
    const options = parseChooseSaveFileOptions(value);
    const dialogOptions: Electron.SaveDialogOptions = {
      title: options.title,
      buttonLabel: options.buttonLabel,
      defaultPath: options.defaultName,
      filters: [{ name: options.filterName, extensions: [...options.extensions] }],
      properties: ['showOverwriteConfirmation'],
    };
    const window = BrowserWindow.fromWebContents(event.sender);
    const result =
      window === null
        ? await dialog.showSaveDialog(dialogOptions)
        : await dialog.showSaveDialog(window, dialogOptions);
    return result.canceled || result.filePath === '' ? null : result.filePath;
  });

  ipcMain.handle(IPC_CHANNELS.openExternal, async (_event, value: unknown) => {
    await shell.openExternal(parseExternalUrl(value));
  });

  ipcMain.handle(IPC_CHANNELS.showInFolder, (_event, value: unknown) => {
    shell.showItemInFolder(parseRevealPath(value));
  });

  ipcMain.handle(IPC_CHANNELS.showNotification, (event, value: unknown) => {
    showSystemNotification(event.sender, parseNotification(value));
  });

  ipcMain.handle(IPC_CHANNELS.resolveFilePath, (_event, value: unknown) => {
    const { folder, fileName } = parseResolveFilePath(value);
    const path = join(folder, fileName);
    return { path, exists: existsSync(path) };
  });

  ipcMain.handle(IPC_CHANNELS.setExportActivity, (_event, value: unknown) => {
    exportGuard.set(parseExportActivity(value));
  });
}

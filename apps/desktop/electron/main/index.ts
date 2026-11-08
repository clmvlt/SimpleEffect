/**
 * Electron main process entry: app lifecycle only (no business logic).
 * It supervises se-core, answers the preload bridge and opens the editor window.
 */
import { app } from 'electron';

import { CORE_PATH_ENV, resolveCoreBinaryPath } from './core-binary-path';
import { CoreSupervisor } from './core-supervisor';
import { registerIpcHandlers } from './ipc-handlers';
import { createMainWindow } from './main-window';
import { notificationAppId } from './platform/app-identity';

const supervisor = new CoreSupervisor({
  binaryPath: resolveCoreBinaryPath({
    isPackaged: app.isPackaged,
    resourcesPath: process.resourcesPath,
    appPath: app.getAppPath(),
    platform: process.platform,
    override: process.env[CORE_PATH_ENV],
  }),
  dataDir: app.getPath('userData'),
});

let coreStopped = false;

const appId = notificationAppId(process.platform);
if (appId !== null) app.setAppUserModelId(appId);

void app.whenReady().then(() => {
  supervisor.start();
  registerIpcHandlers(supervisor);
  createMainWindow();
});

app.on('window-all-closed', () => {
  app.quit();
});

// Stop the core before quitting so it can finish its work; quitting resumes once it has exited.
app.on('before-quit', (event) => {
  if (coreStopped) {
    return;
  }
  event.preventDefault();
  void supervisor.stop().finally(() => {
    coreStopped = true;
    app.quit();
  });
});

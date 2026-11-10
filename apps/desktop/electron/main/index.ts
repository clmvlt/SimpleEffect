/**
 * Electron main process entry: app lifecycle only (no business logic).
 * It supervises se-core, answers the preload bridge and opens the editor window.
 */
import { app, BrowserWindow } from 'electron';

import { CORE_PATH_ENV, resolveCoreBinaryPath } from './core-binary-path';
import { CoreSupervisor } from './core-supervisor';
import { ExportActivityGuard } from './export-activity';
import { electronExportGuardServices } from './export-guard-services';
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

// Exports run in the renderer: closing while one runs asks first (prompt 27).
const exportGuard = new ExportActivityGuard(electronExportGuardServices());

const appId = notificationAppId(process.platform, app.isPackaged, process.execPath);
if (appId !== null) app.setAppUserModelId(appId);

void app.whenReady().then(() => {
  supervisor.start();
  registerIpcHandlers(supervisor, exportGuard);
  const window = createMainWindow();
  window.on('close', (event) => {
    if (!exportGuard.mayLeave(window)) event.preventDefault();
  });
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
  if (!exportGuard.mayLeave(BrowserWindow.getFocusedWindow())) return;
  void supervisor.stop().finally(() => {
    coreStopped = true;
    app.quit();
  });
});

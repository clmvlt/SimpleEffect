import { join } from 'node:path';

import { app, BrowserWindow } from 'electron';

/** Set by `electron-vite dev` to the Vite dev server URL. */
const DEV_SERVER_URL_ENV = 'ELECTRON_RENDERER_URL';

/**
 * Creates the editor window with Electron's hardened defaults: isolated, sandboxed renderer
 * without Node.js, no pop-ups and no navigation away from the app.
 */
export function createMainWindow(): BrowserWindow {
  const window = new BrowserWindow({
    width: 1280,
    height: 800,
    show: false,
    title: 'SimpleEffect',
    webPreferences: {
      preload: join(import.meta.dirname, '../preload/index.cjs'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
  });
  window.once('ready-to-show', () => {
    window.show();
  });
  window.webContents.setWindowOpenHandler(() => ({ action: 'deny' }));
  window.webContents.on('will-navigate', (event) => {
    event.preventDefault();
  });

  const devServerUrl = process.env[DEV_SERVER_URL_ENV];
  if (!app.isPackaged && devServerUrl !== undefined) {
    void window.loadURL(devServerUrl);
  } else {
    void window.loadFile(join(import.meta.dirname, '../renderer/index.html'));
  }
  return window;
}

import { ipcMain } from 'electron';

import { IPC_CHANNELS } from '../shared/bridge-api';
import type { CoreSupervisor } from './core-supervisor';

/** Answers the preload bridge's requests (see `electron/shared/bridge-api.ts`). */
export function registerIpcHandlers(supervisor: CoreSupervisor): void {
  ipcMain.handle(IPC_CHANNELS.getCoreConnection, () => supervisor.waitForConnection());
}

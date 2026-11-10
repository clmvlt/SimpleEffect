import { readFile } from 'node:fs/promises';

import { ipcMain, shell } from 'electron';

import { IPC_CHANNELS, type RuntimeInfo, type ThirdPartyNotices } from '../shared/bridge-api';
import type { LicencePaths } from './resource-paths';
import { parseThirdPartyNotices } from './third-party-notices';

/** What « À propos » asks the main process (prompt 30): runtime versions and licences. */
export function registerAboutHandlers(licences: LicencePaths, packaged: boolean): void {
  ipcMain.handle(IPC_CHANNELS.getRuntimeInfo, (): RuntimeInfo => ({
    electron: process.versions.electron,
    chrome: process.versions.chrome,
    node: process.versions.node,
    v8: process.versions.v8,
    platform: process.platform,
    arch: process.arch,
    packaged,
  }));

  ipcMain.handle(IPC_CHANNELS.getThirdPartyNotices, async (): Promise<ThirdPartyNotices | null> => {
    let json: string;
    try {
      json = await readFile(licences.notices, 'utf8');
    } catch {
      // Not generated yet (`pnpm dev` before any `pnpm build`): the screen says so.
      return null;
    }
    return parseThirdPartyNotices(json);
  });

  ipcMain.handle(IPC_CHANNELS.showLicencesFolder, async () => {
    const problem = await shell.openPath(licences.folder);
    if (problem !== '') throw new Error(`Cannot open ${licences.folder}: ${problem}`);
  });
}

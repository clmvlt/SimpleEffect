import { dialog, powerSaveBlocker } from 'electron';

import type { ExportGuardServices } from './export-activity';

/** Electron's power save blocker and a native confirmation, for the export guard. */
export function electronExportGuardServices(): ExportGuardServices {
  return {
    preventSuspension: () => powerSaveBlocker.start('prevent-app-suspension'),
    allowSuspension: (blockerId) => {
      powerSaveBlocker.stop(blockerId);
    },
    confirmLeave: (window, activity) => {
      const options: Electron.MessageBoxSyncOptions = {
        type: 'warning',
        title: activity.title,
        message: activity.message,
        buttons: [activity.stayLabel, activity.quitLabel],
        defaultId: 0,
        cancelId: 0,
        noLink: true,
      };
      const choice =
        window === null
          ? dialog.showMessageBoxSync(options)
          : dialog.showMessageBoxSync(window, options);
      return choice === 1;
    },
  };
}

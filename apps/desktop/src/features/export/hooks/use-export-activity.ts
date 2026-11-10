import { useEffect, useEffectEvent } from 'react';
import { useTranslation } from 'react-i18next';
import { useStore } from 'zustand';

import { useDesktopBridge } from '../../../shared/desktop';
import { exportsStore, isActiveExport } from '../store/exports-store';

/**
 * Tells the main process how many exports run: while some do, it keeps the computer from
 * sleeping and asks before the window closes or the app quits (the texts are translated here).
 */
export function useExportActivity(): void {
  const bridge = useDesktopBridge();
  const { t } = useTranslation();
  const active = useStore(exportsStore, (state) => state.exports.filter(isActiveExport).length);
  const report = useEffectEvent((count: number) => {
    const activity =
      count === 0
        ? null
        : {
            title: t('export.quit.title', { count }),
            message: t('export.quit.message', { count }),
            quitLabel: t('export.quit.leave'),
            stayLabel: t('export.quit.stay'),
          };
    bridge.setExportActivity(activity).catch((error: unknown) => {
      console.warn('[export] telling the main process about running exports failed', error);
    });
  });
  useEffect(() => {
    report(active);
  }, [active]);
}

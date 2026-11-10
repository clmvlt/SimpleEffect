import { toast } from '@se/ui';
import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';

import { selectProjectId, selectRecovery, useProjectSelector } from '../../shared/project-store';
import { recoveryNotice } from './recovery-notice';

/**
 * Projects whose recovery was already announced in this window: the core keeps the report for
 * as long as the project stays open, and the editor remounts (home and back, reloads of the
 * mirror), so the toast is shown once per project per renderer session.
 */
const announced = new Set<string>();

/** Forgets the announced projects (tests). */
export function resetRecoveryNotices(): void {
  announced.clear();
}

/**
 * Shows once « Projet récupéré après un arrêt inattendu » (journal replayed) or « restauré depuis
 * la sauvegarde » (corrupted `project.json`) when the loaded project was repaired on opening.
 */
export function useRecoveryNotice(): void {
  const { t } = useTranslation();
  const projectId = useProjectSelector(selectProjectId);
  const recovery = useProjectSelector(selectRecovery);

  useEffect(() => {
    const notice = recoveryNotice(recovery);
    if (projectId === null || notice === null || announced.has(projectId)) return;
    announced.add(projectId);
    if (notice.kind === 'replayed') {
      toast.success(t('editor.recovery.replayed', { count: notice.count }));
      return;
    }
    const description =
      notice.lost > 0
        ? t('editor.recovery.lost', { count: notice.lost })
        : notice.replayed > 0
          ? t('editor.recovery.replayedAfterRestore', { count: notice.replayed })
          : t('editor.recovery.setAside');
    toast.warning(t('editor.recovery.restored', { backup: notice.backup }), {
      description,
      duration: Infinity,
    });
  }, [projectId, recovery, t]);
}

import type { Task } from '@se/protocol';
import { toast } from '@se/ui';
import { useEffect, useEffectEvent } from 'react';
import { useTranslation } from 'react-i18next';

import {
  aiTasksStore,
  composerRequestStore,
  openComposer,
  showTask,
  type ComposerRequest,
} from '../../../shared/ai-tasks';
import { useCoreClient } from '../../../shared/core';
import { useDesktopBridge } from '../../../shared/desktop';
import {
  selectProjectId,
  useProjectMirrorReader,
  useProjectSelector,
} from '../../../shared/project-store';
import { useShortcutAction } from '../../../shared/shortcuts';
import { uiPreferencesStorage } from '../../../shared/storage';
import { taskNotice } from '../services/task-notice';
import { connectTaskSync } from '../services/task-sync';
import { redoTargetOf } from '../services/task-scope';
import {
  connectComposerPersistence,
  focusComposer,
  setComposerText,
  startRedo,
} from '../store/composer-store';

/** Whether the app window has the focus (system notifications are for when it has not). */
function windowFocused(): boolean {
  return typeof document === 'undefined' || document.hasFocus();
}

/**
 * Keeps the AI tasks of the open project mirrored for the whole editor (task panel, timeline
 * overlays and pulses), announces the end of each run (a toast with « Voir », plus a system
 * notification when the window is not focused), binds Ctrl/⌘+I to the composer, applies composer
 * requests (« Refaire avec l'IA » on a timeline item) and restores the composer's preferences.
 * Mounted once by the editor layout.
 */
export function useAiTaskSync(): void {
  const client = useCoreClient();
  const bridge = useDesktopBridge();
  const projectId = useProjectSelector(selectProjectId);
  const { t } = useTranslation();
  const readMirror = useProjectMirrorReader();

  const announce = useEffectEvent((task: Task) => {
    const notice = taskNotice(task, aiTasksStore.getState().numbers.get(task.id) ?? 0);
    if (notice === null) return;
    const title = t(notice.tone === 'success' ? 'ai.notice.completed' : 'ai.notice.failed', {
      number: notice.number,
    });
    const description =
      notice.tone === 'success' ? notice.excerpt : t(`ai.failure.${notice.failure}`);
    const action = {
      label: t('ai.notice.show'),
      onClick: () => {
        showTask(task.id);
      },
    };
    if (notice.tone === 'success') toast.success(title, { description, action });
    else toast.error(title, { description, action });
    if (!windowFocused()) {
      bridge
        .showNotification({ title, body: description, tag: task.id })
        .catch((error: unknown) => {
          console.warn('[ai] showing a system notification failed', error);
        });
    }
  });

  const applyRequest = useEffectEvent((request: ComposerRequest) => {
    const mirror = readMirror();
    const item = request.itemId === null ? undefined : mirror?.items[request.itemId];
    const sequenceId = item === undefined ? undefined : mirror?.tracks[item.trackId]?.sequenceId;
    if (item !== undefined && sequenceId !== undefined) startRedo(redoTargetOf(item, sequenceId));
    else focusComposer();
    if (request.draft !== null) setComposerText(request.draft);
  });

  useEffect(
    () =>
      composerRequestStore.subscribe(({ request }, previous) => {
        if (request !== null && request !== previous.request) applyRequest(request);
      }),
    [],
  );
  useEffect(() => {
    if (projectId === null) return;
    return connectTaskSync(client, projectId, { onRunFinished: announce });
  }, [client, projectId]);
  useEffect(
    () =>
      bridge.onNotificationClick((tag) => {
        // Other features tag their notifications too (exports: `export:<id>`).
        if (aiTasksStore.getState().tasks.some((task) => task.id === tag)) showTask(tag);
      }),
    [bridge],
  );
  useEffect(() => connectComposerPersistence(uiPreferencesStorage()), []);
  useShortcutAction('askAi', () => {
    openComposer();
  });
}

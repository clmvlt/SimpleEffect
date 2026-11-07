import { toast } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../shared/core';
import { errorMessageKey } from '../../shared/errors';
import { selectHistory, useProjectSelector } from '../../shared/project-store';
import { useShortcutAction } from '../../shared/shortcuts';

/**
 * Undo and redo of the open project, bound to the `undo`/`redo` shortcut actions (Ctrl/⌘+Z,
 * Ctrl/⌘+Maj+Z, Ctrl+Y off Apple). The core does the work; the result arrives as a patch.
 */
export function useHistoryCommands(projectId: string) {
  const client = useCoreClient();
  const history = useProjectSelector(selectHistory);
  const { t } = useTranslation();
  const run = (kind: 'undo' | 'redo') => {
    client.send({ kind, projectId }).catch((error: unknown) => {
      toast.error(t(errorMessageKey(error)));
    });
  };
  const undo = () => {
    if (history?.canUndo === true) run('undo');
  };
  const redo = () => {
    if (history?.canRedo === true) run('redo');
  };
  useShortcutAction('undo', undo);
  useShortcutAction('redo', redo);
  return { history, undo, redo };
}

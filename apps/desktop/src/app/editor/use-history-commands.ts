import { toast } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../shared/core';
import { errorMessageKey } from '../../shared/errors';
import { selectHistory, useProjectSelector } from '../../shared/project-store';
import { useShortcut } from '../../shared/shortcuts';

/**
 * Undo and redo of the open project, bound to Ctrl/⌘+Z and Ctrl/⌘+Maj+Z. The core does the
 * work; the result arrives as a patch.
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
  useShortcut('mod+z', undo);
  useShortcut('mod+shift+z', redo);
  return { history, undo, redo };
}

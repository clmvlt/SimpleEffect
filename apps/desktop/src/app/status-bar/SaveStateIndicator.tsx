import { StatusDot, Tooltip } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { selectSaveState, useProjectSelector } from '../../shared/project-store';

/**
 * The open project's save failure in the status bar (prompt 29): nothing while saving works;
 * when the core cannot write the project (disk full, folder gone, permissions), a red « Sauvegarde
 * impossible » with the unsaved changes, the core's actionable message in the tooltip. Edits stay
 * in the journal and the core retries at the next change, so the alert clears by itself.
 */
export function SaveStateIndicator() {
  const { t } = useTranslation();
  const saveState = useProjectSelector(selectSaveState);
  if (saveState?.failure == null) return null;
  return (
    <Tooltip content={saveState.failure} side="top" align="start">
      <p
        role="alert"
        tabIndex={0}
        data-save-failure=""
        className="flex h-5 items-center gap-1.5 rounded-sm px-1 text-danger focus-visible:outline-1 focus-visible:outline-accent"
      >
        <StatusDot tone="danger" size="xs" />
        {t('statusBar.save.failed', { count: saveState.unsavedChanges })}
      </p>
    </Tooltip>
  );
}

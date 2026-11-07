import { Button, Icon, IconButton, Separator } from '@se/ui';
import { House, Redo2, Undo2 } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import {
  selectActiveSequence,
  selectProjectName,
  useProjectSelector,
} from '../../shared/project-store';
import { useHistoryCommands } from './use-history-commands';

/** Props of {@link EditorHeader}. */
export interface EditorHeaderProps {
  readonly projectId: string;
  readonly onHome: () => void;
}

/** The editor's top bar: back to the home screen, project and sequence names, undo and redo. */
export function EditorHeader({ projectId, onHome }: EditorHeaderProps) {
  const { t } = useTranslation();
  const name = useProjectSelector(selectProjectName);
  const sequence = useProjectSelector(selectActiveSequence);
  const { history, undo, redo } = useHistoryCommands(projectId);
  const undoLabel = history?.undoLabel ?? null;
  const redoLabel = history?.redoLabel ?? null;
  return (
    <header
      aria-label={t('editor.header.label')}
      className="flex h-10 shrink-0 items-center gap-2 border-b border-border bg-panel px-2"
    >
      <Button variant="ghost" size="sm" onClick={onHome}>
        <Icon icon={House} size="sm" />
        {t('editor.backHome')}
      </Button>
      <Separator orientation="vertical" className="h-5" />
      <h1 className="truncate text-md font-semibold text-text">{name}</h1>
      {sequence !== null && (
        <span className="truncate text-sm text-text-muted">
          {t('editor.header.sequence', { name: sequence.name })}
        </span>
      )}
      <div className="ml-auto flex items-center gap-1">
        <IconButton
          icon={Undo2}
          label={
            undoLabel === null
              ? t('editor.header.undo')
              : t('editor.header.undoNamed', { label: undoLabel })
          }
          disabled={history?.canUndo !== true}
          onClick={undo}
        />
        <IconButton
          icon={Redo2}
          label={
            redoLabel === null
              ? t('editor.header.redo')
              : t('editor.header.redoNamed', { label: redoLabel })
          }
          disabled={history?.canRedo !== true}
          onClick={redo}
        />
      </div>
    </header>
  );
}

import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  IconButton,
} from '@se/ui';
import { Ellipsis } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import type { TaskActions } from '../services/task-log';

/** What the buttons of a task card do. */
export interface TaskActionHandlers {
  readonly cancel: () => void;
  readonly retry: () => void;
  readonly followUp: () => void;
  readonly revert: () => void;
  readonly showChanges: () => void;
  readonly showLog: () => void;
}

/** Props of {@link TaskActionButtons}. */
export interface TaskActionButtonsProps {
  readonly actions: TaskActions;
  readonly on: TaskActionHandlers;
  /** The inline follow-up field is open. */
  readonly followingUp: boolean;
}

/**
 * The buttons of a task card: « Annuler » while it waits or runs, « Relancer », « Suite » and
 * « Annuler les modifications » once finished, and a menu for « Voir les modifications » and the
 * complete log.
 */
export function TaskActionButtons({ actions, on, followingUp }: TaskActionButtonsProps) {
  const { t } = useTranslation();
  return (
    <>
      {actions.cancel && (
        <Button size="xs" variant="ghost" onClick={on.cancel}>
          {t('ai.tasks.cancel')}
        </Button>
      )}
      {actions.retry && (
        <Button size="xs" variant="ghost" onClick={on.retry}>
          {t('ai.tasks.retry')}
        </Button>
      )}
      {actions.followUp && (
        <Button size="xs" variant="ghost" aria-expanded={followingUp} onClick={on.followUp}>
          {t('ai.tasks.followUp')}
        </Button>
      )}
      {actions.revert && (
        <Button size="xs" variant="ghost" onClick={on.revert}>
          {t('ai.tasks.revert')}
        </Button>
      )}
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <IconButton icon={Ellipsis} size="xs" label={t('ai.tasks.more')} />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <DropdownMenuItem disabled={!actions.changes} onSelect={on.showChanges}>
            {t('ai.tasks.showChanges')}
          </DropdownMenuItem>
          <DropdownMenuItem onSelect={on.showLog}>{t('ai.tasks.showLog')}</DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </>
  );
}

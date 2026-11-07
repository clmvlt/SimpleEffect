import type { TransitionAlignment, TransitionId } from '@se/protocol';
import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuRadioGroup,
  ContextMenuRadioItem,
  ContextMenuSeparator,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import type { TransitionActions } from '../hooks/use-transition-actions';

/** Props of {@link TransitionMenu}. */
export interface TransitionMenuProps {
  readonly transitionId: TransitionId;
  readonly alignment: TransitionAlignment;
  readonly actions: TransitionActions;
}

const ALIGNMENTS: readonly TransitionAlignment[] = ['center', 'start', 'end'];

function isAlignment(value: string): value is TransitionAlignment {
  return (ALIGNMENTS as readonly string[]).includes(value);
}

/** The context menu of a transition: where it sits on its cut, and removing it. */
export function TransitionMenu({ transitionId, alignment, actions }: TransitionMenuProps) {
  const { t } = useTranslation();
  return (
    <ContextMenuContent>
      <ContextMenuRadioGroup
        value={alignment}
        onValueChange={(value) => {
          if (isAlignment(value) && value !== alignment) actions.realign(transitionId, value);
        }}
      >
        {ALIGNMENTS.map((option) => (
          <ContextMenuRadioItem key={option} value={option}>
            {t(`editor.timeline.transition.alignments.${option}`)}
          </ContextMenuRadioItem>
        ))}
      </ContextMenuRadioGroup>
      <ContextMenuSeparator />
      <ContextMenuItem
        variant="danger"
        onSelect={() => {
          actions.remove(transitionId);
        }}
      >
        {t('editor.timeline.transition.remove')}
      </ContextMenuItem>
    </ContextMenuContent>
  );
}

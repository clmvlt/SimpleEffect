import type { Flicks, ItemEntity, TransitionAlignment, TransitionId } from '@se/protocol';
import { ContextMenuContent, ContextMenuItem, ContextMenuSeparator } from '@se/ui';
import { useTranslation } from 'react-i18next';

import type { EditActions } from '../hooks/use-edit-actions';
import type { TransitionActions } from '../hooks/use-transition-actions';
import type { Gap } from '../services/gaps';
import { canLink, canUnlink, crosses } from '../services/item-edits';
import { TransitionMenu } from './TransitionMenu';

/** What a right click in the lanes landed on. */
export type LaneMenuTarget =
  | { readonly kind: 'item'; readonly item: ItemEntity; readonly time: Flicks }
  | { readonly kind: 'gap'; readonly gap: Gap }
  | {
      readonly kind: 'transition';
      readonly transitionId: TransitionId;
      readonly alignment: TransitionAlignment;
    }
  | { readonly kind: 'empty' };

/** Props of {@link LaneMenu}. */
export interface LaneMenuProps {
  readonly target: LaneMenuTarget;
  readonly actions: EditActions;
  readonly transitionActions: TransitionActions;
  /** Whether the clipboard holds items of this project. */
  readonly canPaste: boolean;
}

/** The asset an item plays, for « Afficher dans le panneau Médias ». */
function assetOf(item: ItemEntity): string | null {
  return item.kind.kind === 'mediaClip' || item.kind.kind === 'image' ? item.kind.assetId : null;
}

/** The context menu of the lanes: item actions, closing a gap, a transition's, or pasting. */
export function LaneMenu({ target, actions, transitionActions, canPaste }: LaneMenuProps) {
  const { t } = useTranslation();
  if (target.kind === 'transition') {
    return (
      <TransitionMenu
        transitionId={target.transitionId}
        alignment={target.alignment}
        actions={transitionActions}
      />
    );
  }
  if (target.kind === 'gap') {
    return (
      <ContextMenuContent>
        <ContextMenuItem
          onSelect={() => {
            actions.deleteSelection(true);
          }}
        >
          {t('editor.timeline.menu.closeGap')}
        </ContextMenuItem>
      </ContextMenuContent>
    );
  }
  if (target.kind === 'empty') {
    return (
      <ContextMenuContent>
        <ContextMenuItem
          disabled={!canPaste}
          onSelect={() => {
            actions.paste('overwrite');
          }}
        >
          {t('editor.timeline.menu.paste')}
        </ContextMenuItem>
      </ContextMenuContent>
    );
  }
  const { item, time } = target;
  const selected = actions.selectedItems();
  const assetId = assetOf(item);
  return (
    <ContextMenuContent>
      <ContextMenuItem
        disabled={!crosses(item, time)}
        onSelect={() => {
          actions.splitAt(time);
        }}
      >
        {t('editor.timeline.menu.splitHere')}
      </ContextMenuItem>
      <ContextMenuItem onSelect={actions.duplicateSelection}>
        {t('editor.timeline.menu.duplicate')}
      </ContextMenuItem>
      <ContextMenuSeparator />
      {canUnlink(selected) ? (
        <ContextMenuItem onSelect={actions.unlinkSelection}>
          {t('editor.timeline.menu.unlink')}
        </ContextMenuItem>
      ) : (
        <ContextMenuItem disabled={!canLink(selected)} onSelect={actions.linkSelection}>
          {t('editor.timeline.menu.link')}
        </ContextMenuItem>
      )}
      <ContextMenuItem onSelect={actions.showProperties}>
        {t('editor.timeline.menu.properties')}
      </ContextMenuItem>
      {assetId !== null && (
        <ContextMenuItem
          onSelect={() => {
            actions.revealInMediaBin(assetId);
          }}
        >
          {t('editor.timeline.menu.revealInMedia')}
        </ContextMenuItem>
      )}
      <ContextMenuSeparator />
      <ContextMenuItem
        variant="danger"
        onSelect={() => {
          actions.deleteSelection(false);
        }}
      >
        {t('editor.timeline.menu.delete')}
      </ContextMenuItem>
      <ContextMenuItem
        variant="danger"
        onSelect={() => {
          actions.deleteSelection(true);
        }}
      >
        {t('editor.timeline.menu.rippleDelete')}
      </ContextMenuItem>
    </ContextMenuContent>
  );
}

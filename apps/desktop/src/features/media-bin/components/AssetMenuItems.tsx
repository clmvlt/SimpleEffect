import type { Asset } from '@se/protocol';
import {
  ContextMenuItem,
  ContextMenuSeparator,
  DropdownMenuItem,
  DropdownMenuSeparator,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import { canGenerateProxy } from '../services/derived-display';

/** What the asset menus do. */
export interface AssetMenuHandlers {
  readonly onDetails: (asset: Asset) => void;
  readonly onRelink: (asset: Asset) => void;
  readonly onReveal: (asset: Asset) => void;
  readonly onGenerateProxy: (asset: Asset) => void;
  readonly onRemove: (asset: Asset) => void;
  /** Adds an image at the playhead (images only; hidden when absent). */
  readonly onAddImage?: ((asset: Asset) => void) | undefined;
}

/** Props of {@link AssetMenuItems}. */
export interface AssetMenuItemsProps extends AssetMenuHandlers {
  readonly asset: Asset;
  /** Which menu hosts the items (context menu on right click, dropdown on the « ⋯ » button). */
  readonly menu: 'context' | 'dropdown';
}

/** Items shared by the asset's context menu and « ⋯ » menu. */
export function AssetMenuItems({ asset, menu, ...handlers }: AssetMenuItemsProps) {
  const { t } = useTranslation();
  const Item = menu === 'context' ? ContextMenuItem : DropdownMenuItem;
  const Separator = menu === 'context' ? ContextMenuSeparator : DropdownMenuSeparator;
  const { onAddImage } = handlers;
  return (
    <>
      {asset.kind === 'image' && onAddImage !== undefined && (
        <>
          <Item
            onSelect={() => {
              onAddImage(asset);
            }}
          >
            {t('editor.media.menu.addAtPlayhead')}
          </Item>
          <Separator />
        </>
      )}
      <Item
        onSelect={() => {
          handlers.onDetails(asset);
        }}
      >
        {t('editor.media.menu.details')}
      </Item>
      <Item
        onSelect={() => {
          handlers.onRelink(asset);
        }}
      >
        {t('editor.media.menu.relink')}
      </Item>
      <Item
        disabled={asset.missing}
        onSelect={() => {
          handlers.onReveal(asset);
        }}
      >
        {t('editor.media.menu.reveal')}
      </Item>
      {asset.kind === 'video' && (
        <Item
          disabled={!canGenerateProxy(asset)}
          onSelect={() => {
            handlers.onGenerateProxy(asset);
          }}
        >
          {t('editor.media.menu.generateProxy')}
        </Item>
      )}
      <Separator />
      <Item
        variant="danger"
        onSelect={() => {
          handlers.onRemove(asset);
        }}
      >
        {t('editor.media.menu.remove')}
      </Item>
    </>
  );
}

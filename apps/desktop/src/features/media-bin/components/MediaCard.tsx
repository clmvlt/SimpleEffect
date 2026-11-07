import type { Asset } from '@se/protocol';
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuTrigger,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuTrigger,
  IconButton,
  cn,
  focusRing,
} from '@se/ui';
import { Ellipsis } from '@se/ui/icons';
import { useTranslation } from 'react-i18next';

import { formatDuration } from '../../../shared/time';
import { useAssetEntry } from '../hooks/use-asset-entry';
import { assetName } from '../services/asset-list';
import { AssetMenuItems, type AssetMenuHandlers } from './AssetMenuItems';
import { AssetPreview } from './AssetPreview';
import { JobIndicator } from './JobIndicator';
import { PlaybackBadge } from './PlaybackBadge';

/** Width of a card's preview in pixels. */
const PREVIEW_WIDTH = 144;

/** Props of {@link MediaCard} and {@link MediaRow}. */
export interface MediaItemProps extends AssetMenuHandlers {
  readonly asset: Asset;
  readonly selected: boolean;
}

/** An asset in the grid: preview (hover scrubs videos), name, duration, state and menus. */
export function MediaCard({ asset, selected, ...handlers }: MediaItemProps) {
  const { t } = useTranslation();
  const name = assetName(asset);
  const duration = asset.mediaInfo?.duration ?? null;
  const entry = useAssetEntry(asset, selected);
  return (
    <li className="relative">
      <ContextMenu>
        <ContextMenuTrigger asChild>
          <button
            {...entry}
            type="button"
            aria-label={t('editor.media.card.open', { name })}
            aria-pressed={selected}
            onClick={() => {
              handlers.onDetails(asset);
            }}
            className={cn(
              'flex w-full flex-col gap-1 rounded-md border border-transparent p-1 text-left',
              'transition-colors duration-(--duration-fast) hover:bg-hover',
              selected && 'border-accent bg-hover',
              focusRing,
            )}
          >
            <span className={cn('relative', asset.missing && 'opacity-50')}>
              <AssetPreview asset={asset} width={PREVIEW_WIDTH} scrub />
              {duration !== null && (
                <span className="absolute right-1 bottom-1 rounded-sm bg-scrim px-1 text-2xs text-text timecode">
                  {formatDuration(duration)}
                </span>
              )}
            </span>
            <span className="flex min-w-0 items-center gap-1">
              <span className="truncate text-xs text-text" title={asset.originalPath}>
                {name}
              </span>
              <PlaybackBadge asset={asset} />
            </span>
            <JobIndicator asset={asset} />
          </button>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <AssetMenuItems asset={asset} menu="context" {...handlers} />
        </ContextMenuContent>
      </ContextMenu>
      <DropdownMenu>
        <DropdownMenuTrigger asChild>
          <IconButton
            icon={Ellipsis}
            size="xs"
            variant="secondary"
            label={t('editor.media.card.actions', { name })}
            className="absolute top-2 right-2"
          />
        </DropdownMenuTrigger>
        <DropdownMenuContent align="end">
          <AssetMenuItems asset={asset} menu="dropdown" {...handlers} />
        </DropdownMenuContent>
      </DropdownMenu>
    </li>
  );
}

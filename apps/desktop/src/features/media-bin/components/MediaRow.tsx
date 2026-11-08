import { ContextMenu, ContextMenuContent, ContextMenuTrigger, cn, focusRing } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { formatDuration } from '../../../shared/time';
import { useAssetEntry } from '../hooks/use-asset-entry';
import { assetName } from '../services/asset-list';
import { AssetMenuItems } from './AssetMenuItems';
import { AssetPreview } from './AssetPreview';
import { JobIndicator } from './JobIndicator';
import type { MediaItemProps } from './MediaCard';
import { PlaybackBadge } from './PlaybackBadge';
import { TranscriptBadge } from './TranscriptBadge';

/** An asset in the list view: small preview, name, kind, duration and state. */
export function MediaRow({ asset, selected, ...handlers }: MediaItemProps) {
  const { t } = useTranslation();
  const name = assetName(asset);
  const duration = asset.mediaInfo?.duration ?? null;
  const entry = useAssetEntry(asset, selected);
  return (
    <li>
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
              'flex w-full items-center gap-2 rounded-sm px-1 py-0.5 text-left hover:bg-hover',
              selected && 'bg-hover',
              focusRing,
            )}
          >
            <span className={cn(asset.missing && 'opacity-50')}>
              <AssetPreview asset={asset} width={48} />
            </span>
            <span className="flex min-w-0 flex-1 flex-col gap-0.5">
              <span className="truncate text-xs text-text" title={asset.originalPath}>
                {name}
              </span>
              <span className="flex items-center gap-1 text-2xs text-text-muted">
                {t(`editor.media.kinds.${asset.kind}`)}
                {duration !== null && <span className="timecode">{formatDuration(duration)}</span>}
                <PlaybackBadge asset={asset} />
                <TranscriptBadge asset={asset} />
              </span>
              <JobIndicator asset={asset} />
            </span>
          </button>
        </ContextMenuTrigger>
        <ContextMenuContent>
          <AssetMenuItems asset={asset} menu="context" {...handlers} />
        </ContextMenuContent>
      </ContextMenu>
    </li>
  );
}

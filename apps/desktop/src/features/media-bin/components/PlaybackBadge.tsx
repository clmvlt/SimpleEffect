import { playbackSource } from '@se/engine/sources';
import type { Asset } from '@se/protocol';
import { Badge } from '@se/ui';
import { useTranslation } from 'react-i18next';

/** Props of {@link PlaybackBadge}. */
export interface PlaybackBadgeProps {
  readonly asset: Asset;
}

/**
 * Why an asset cannot be played yet: « Manquant » (original not found), « En préparation »
 * (mezzanine or decodability check pending), « Illisible » (its mezzanine failed). Nothing for a
 * playable asset.
 */
export function PlaybackBadge({ asset }: PlaybackBadgeProps) {
  const { t } = useTranslation();
  const source = playbackSource(asset);
  switch (source.kind) {
    case 'original':
    case 'mezzanine':
      return null;
    case 'preparing':
      return (
        <Badge variant="info" size="xs">
          {t('editor.media.card.preparing')}
        </Badge>
      );
    case 'unavailable':
      return (
        <Badge variant="warning" size="xs">
          {source.reason === 'missing'
            ? t('editor.media.card.missing')
            : t('editor.media.card.unplayable')}
        </Badge>
      );
  }
}

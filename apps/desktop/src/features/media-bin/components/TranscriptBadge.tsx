import type { Asset } from '@se/protocol';
import { Badge } from '@se/ui';
import { useTranslation } from 'react-i18next';

/** Props of {@link TranscriptBadge}. */
export interface TranscriptBadgeProps {
  readonly asset: Asset;
}

/** « Transcrit » on media whose sound is transcribed (its model and word count as a tooltip). */
export function TranscriptBadge({ asset }: TranscriptBadgeProps) {
  const { t } = useTranslation();
  const summary = asset.analysis.transcript;
  if (summary === null) return null;
  return (
    <Badge
      variant="caption"
      size="xs"
      title={t('editor.media.card.transcribedDetails', {
        model: summary.model,
        count: summary.wordCount,
      })}
    >
      {t('editor.media.card.transcribed')}
    </Badge>
  );
}

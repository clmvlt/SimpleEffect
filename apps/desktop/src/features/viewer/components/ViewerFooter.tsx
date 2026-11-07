import type { SequenceEntity } from '@se/protocol';
import { TimecodeDisplay } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { formatTimecode } from '../../../shared/time';
import { usePlayheadTime } from '../../timeline';

/** Props of {@link ViewerFooter}. */
export interface ViewerFooterProps {
  readonly sequence: SequenceEntity;
}

/** The current timecode and the sequence format under the picture. */
export function ViewerFooter({ sequence }: ViewerFooterProps) {
  const { t, i18n } = useTranslation();
  const { width, height, frameRate } = sequence.format;
  const time = usePlayheadTime(sequence.id);
  const fps = new Intl.NumberFormat(i18n.language, { maximumFractionDigits: 3 }).format(
    frameRate.numerator / frameRate.denominator,
  );
  return (
    <div className="flex h-control-sm shrink-0 items-center justify-between gap-2 border-t border-border px-2 text-xs text-text-muted">
      <TimecodeDisplay
        className="text-text"
        label={t('editor.viewer.playhead')}
        value={time}
        format={(value) => formatTimecode(value as typeof time, frameRate)}
      />
      <span className="truncate">{t('editor.viewer.format', { width, height, fps })}</span>
    </div>
  );
}

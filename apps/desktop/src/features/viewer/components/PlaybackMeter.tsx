import { Meter } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useMeterLevels } from '../hooks/use-meter-levels';
import type { PlaybackSession } from '../services/playback-session';

/** Props of {@link PlaybackMeter}. */
export interface PlaybackMeterProps {
  readonly session: PlaybackSession;
}

/** The master level meter of the transport bar (RMS bars, held peaks), refreshed on its own. */
export function PlaybackMeter({ session }: PlaybackMeterProps) {
  const { t } = useTranslation();
  const { levels, peaks } = useMeterLevels(session);
  return (
    <Meter
      orientation="horizontal"
      showScale={false}
      className="w-24 min-w-24"
      levels={levels}
      peaks={peaks}
      label={t('editor.viewer.meter.label')}
      channelLabels={[t('editor.viewer.meter.left'), t('editor.viewer.meter.right')]}
    />
  );
}

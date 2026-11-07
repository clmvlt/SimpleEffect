import { ScrubInput, Tooltip, ToolbarButton } from '@se/ui';
import { AudioWaveform } from '@se/ui/icons';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import {
  TRANSITION_SECONDS_RANGE,
  setEditPreferences,
  useEditPreferences,
} from '../store/edit-preferences-store';

/** Seconds with a decimal comma (`1,5 s`). */
function formatSeconds(value: number): string {
  return `${value.toFixed(1).replace('.', ',')} s`;
}

/**
 * The sound and transition tools of the timeline toolbar: the length of new transitions
 * (`Ctrl/⌘+T`, a click on a cut) and the silences shown on audio clips. Both are machine
 * preferences.
 */
export function TimelineSoundTools() {
  const { t } = useTranslation();
  const { transitionSeconds, showSilences } = useEditPreferences();
  const [draft, setDraft] = useState<number | null>(null);
  return (
    <>
      <Tooltip content={t('editor.timeline.toolbar.transitionDurationHint')}>
        <ScrubInput
          size="xs"
          className="w-20"
          label={t('editor.timeline.toolbar.transitionDuration')}
          value={draft ?? transitionSeconds}
          min={TRANSITION_SECONDS_RANGE.min}
          max={TRANSITION_SECONDS_RANGE.max}
          step={0.1}
          format={formatSeconds}
          onChange={setDraft}
          onCommit={(value) => {
            setDraft(null);
            setEditPreferences({ transitionSeconds: value });
          }}
        />
      </Tooltip>
      <Tooltip content={t('editor.timeline.toolbar.silences')}>
        <ToolbarButton
          icon={AudioWaveform}
          label={t('editor.timeline.toolbar.silences')}
          pressed={showSilences}
          onClick={() => {
            setEditPreferences({ showSilences: !showSilences });
          }}
        />
      </Tooltip>
    </>
  );
}

import type { Flicks, SilenceCutParams } from '@se/protocol';
import { Label, Slider } from '@se/ui';
import { useId } from 'react';
import { useTranslation } from 'react-i18next';

import { FLICKS_PER_SECOND, flicksToSeconds } from '../../../shared/time';
import { cutSeconds } from '../services/cut-format';

/** Props of {@link SilenceParamFields}. */
export interface SilenceParamFieldsProps {
  readonly params: SilenceCutParams;
  readonly onChange: (params: SilenceCutParams) => void;
}

/** Seconds from a slider as flicks (slider values are small and exact enough). */
function toFlicks(seconds: number): Flicks {
  return Math.round(seconds * FLICKS_PER_SECOND) as Flicks;
}

interface SliderRowProps {
  readonly label: string;
  readonly value: number;
  readonly shown: string;
  readonly min: number;
  readonly max: number;
  readonly step: number;
  readonly onChange: (value: number) => void;
}

function SliderRow({ label, value, shown, min, max, step, onChange }: SliderRowProps) {
  const id = useId();
  return (
    <div className="flex flex-col gap-1">
      <div className="flex items-center justify-between">
        <Label id={id} className="text-xs">
          {label}
        </Label>
        <span className="text-2xs text-text-muted tabular-nums">{shown}</span>
      </div>
      <Slider
        aria-labelledby={id}
        thumbLabels={[label]}
        min={min}
        max={max}
        step={step}
        value={[value]}
        onValueChange={([next]) => {
          if (next !== undefined) onChange(next);
        }}
      />
    </div>
  );
}

/** Threshold (dBFS), shortest silence and padding of a silence removal. */
export function SilenceParamFields({ params, onChange }: SilenceParamFieldsProps) {
  const { t } = useTranslation();
  const seconds = (flicks: Flicks) => flicksToSeconds(flicks);
  return (
    <div className="flex flex-col gap-3">
      <SliderRow
        label={t('editor.transcript.silences.threshold')}
        value={params.thresholdDbfs}
        shown={t('editor.transcript.silences.dbfs', { value: params.thresholdDbfs })}
        min={-80}
        max={-10}
        step={1}
        onChange={(thresholdDbfs) => {
          onChange({ ...params, thresholdDbfs });
        }}
      />
      <SliderRow
        label={t('editor.transcript.silences.minSilence')}
        value={seconds(params.minSilence)}
        shown={t('editor.transcript.silences.seconds', { value: cutSeconds(params.minSilence) })}
        min={0.1}
        max={3}
        step={0.05}
        onChange={(value) => {
          onChange({ ...params, minSilence: toFlicks(value) });
        }}
      />
      <SliderRow
        label={t('editor.transcript.silences.padding')}
        value={seconds(params.padding)}
        shown={t('editor.transcript.silences.paddingValue', {
          value: Math.round(seconds(params.padding) * 1000),
        })}
        min={0}
        max={0.5}
        step={0.01}
        onChange={(value) => {
          onChange({ ...params, padding: toFlicks(value) });
        }}
      />
    </div>
  );
}

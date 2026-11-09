import type { TrackingSettings } from '@se/protocol';
import { PropertyRow, Slider, Switch } from '@se/ui';
import { useId } from 'react';
import { useTranslation } from 'react-i18next';

import { TRACKING_RANGES, withSetting, type NumericTrackingSetting } from '../services/tracking';

/** Props of {@link TrackingSettingsFields}. */
export interface TrackingSettingsFieldsProps {
  readonly settings: TrackingSettings;
  /** Every change, while dragging too. */
  readonly onChange: (settings: TrackingSettings) => void;
  /** A finished change (slider released, switch toggled); applied at once when given. */
  readonly onCommit?: ((settings: TrackingSettings) => void) | undefined;
  readonly disabled?: boolean;
}

const SLIDERS: readonly NumericTrackingSetting[] = ['smoothing', 'eyeLine', 'deadZone'];

/** « Fluidité », « Ligne des yeux », « Zone morte » and « Cadrage de groupe » of tracking. */
export function TrackingSettingsFields({
  settings,
  onChange,
  onCommit,
  disabled = false,
}: TrackingSettingsFieldsProps) {
  const { t } = useTranslation();
  const groupId = useId();
  return (
    <div className="flex flex-col">
      {SLIDERS.map((setting) => {
        const { min, max } = TRACKING_RANGES[setting];
        const label = t(`editor.reframing.settings.${setting}`);
        return (
          <PropertyRow key={setting} label={label}>
            <div className="flex w-full items-center gap-2">
              <Slider
                thumbLabels={[label]}
                min={min}
                max={max}
                step={0.01}
                disabled={disabled}
                value={[settings[setting]]}
                onValueChange={([value]) => {
                  if (value !== undefined) onChange(withSetting(settings, setting, value));
                }}
                onValueCommit={([value]) => {
                  if (value !== undefined) onCommit?.(withSetting(settings, setting, value));
                }}
              />
              <span className="w-9 shrink-0 text-right text-xs text-text-muted tabular-nums">
                {t('editor.reframing.settings.percent', {
                  value: Math.round(settings[setting] * 100),
                })}
              </span>
            </div>
          </PropertyRow>
        );
      })}
      <PropertyRow label={t('editor.reframing.settings.groupFraming')} htmlFor={groupId}>
        <Switch
          id={groupId}
          disabled={disabled}
          checked={settings.groupFraming}
          onCheckedChange={(checked) => {
            const next = { ...settings, groupFraming: checked };
            onChange(next);
            onCommit?.(next);
          }}
        />
      </PropertyRow>
    </div>
  );
}

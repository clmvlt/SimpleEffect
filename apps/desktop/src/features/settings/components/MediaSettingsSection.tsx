import type { MediaSettings, ProxyResolution } from '@se/protocol';
import { Field, Slider, Switch, ToggleGroup, ToggleGroupItem } from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

/** Accepted mezzanine qualities (CRF), as in `MediaSettings::MEZZANINE_QUALITY_RANGE`. */
const QUALITY_MIN = 8;
const QUALITY_MAX = 28;
const RESOLUTIONS: readonly ProxyResolution[] = ['540p', '720p'];

/** Props of {@link MediaSettingsSection}. */
export interface MediaSettingsSectionProps {
  readonly settings: MediaSettings;
  readonly onSave: (changes: Partial<MediaSettings>) => void;
}

/** Mezzanine quality, proxy resolution and automatic proxies. */
export function MediaSettingsSection({ settings, onSave }: MediaSettingsSectionProps) {
  const { t } = useTranslation();
  const [quality, setQuality] = useState<number | null>(null);
  const shownQuality = quality ?? settings.mezzanineQuality;
  return (
    <section aria-labelledby="settings-derived" className="flex flex-col gap-4">
      <h3 id="settings-derived" className="text-sm font-semibold text-text">
        {t('settings.media.derivedTitle')}
      </h3>
      <div className="flex flex-col gap-1.5">
        <div className="flex items-center justify-between text-xs">
          <span className="font-medium text-text">{t('settings.media.quality')}</span>
          <span className="text-text-muted timecode">
            {t('settings.media.qualityValue', { value: shownQuality })}
          </span>
        </div>
        <Slider
          thumbLabels={[t('settings.media.quality')]}
          min={QUALITY_MIN}
          max={QUALITY_MAX}
          step={1}
          value={[shownQuality]}
          onValueChange={([value]) => {
            if (value !== undefined) setQuality(value);
          }}
          onValueCommit={([value]) => {
            setQuality(null);
            if (value !== undefined) onSave({ mezzanineQuality: value });
          }}
        />
        <p className="text-2xs text-text-muted">{t('settings.media.qualityDescription')}</p>
      </div>
      <Field label={t('settings.media.proxyResolution')}>
        <ToggleGroup
          type="single"
          variant="segmented"
          value={settings.proxyResolution}
          onValueChange={(value) => {
            const resolution = RESOLUTIONS.find((candidate) => candidate === value);
            if (resolution !== undefined) onSave({ proxyResolution: resolution });
          }}
        >
          {RESOLUTIONS.map((resolution) => (
            <ToggleGroupItem key={resolution} value={resolution}>
              {resolution}
            </ToggleGroupItem>
          ))}
        </ToggleGroup>
      </Field>
      <Field
        label={t('settings.media.autoProxies')}
        description={t('settings.media.autoProxiesDescription')}
        orientation="horizontal"
      >
        <Switch
          checked={settings.autoProxies}
          onCheckedChange={(checked) => {
            onSave({ autoProxies: checked });
          }}
        />
      </Field>
      <p className="text-2xs text-text-muted">{t('settings.media.appliesToNew')}</p>
    </section>
  );
}

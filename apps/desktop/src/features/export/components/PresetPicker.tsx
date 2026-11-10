import type { ExportSettings } from '@se/protocol';
import {
  Button,
  Field,
  Input,
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectLabel,
  SelectSeparator,
  SelectTrigger,
  SelectValue,
} from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import type { ExportForm } from '../hooks/use-export-form';
import { isMasterCodec } from '../services/export-draft';
import { BUILT_IN_PRESETS, choiceValue, parseChoice } from '../services/export-presets';

/** One line describing `settings`: size, codecs and rates (« 1920 × 1080 · H.264 · 12 Mb/s »). */
export function SettingsSummary({ settings }: { readonly settings: ExportSettings }) {
  const { t } = useTranslation();
  const parts: string[] = [t(`export.containers.${settings.container}`)];
  const { video, audio } = settings;
  if (video !== null) {
    parts.push(t('export.summary.size', { width: video.width, height: video.height }));
    parts.push(t(`export.codecs.${video.codec}`));
    if (!isMasterCodec(video.codec)) {
      parts.push(
        video.rate.kind === 'bitrate'
          ? t('export.summary.megabits', { value: video.rate.bitsPerSecond / 1_000_000 })
          : t(`export.quality.${video.rate.level}`),
      );
    }
  }
  if (audio !== null) {
    parts.push(
      audio.codec === 'pcm'
        ? t('export.audioCodecs.pcm')
        : t('export.summary.audio', {
            codec: t(`export.audioCodecs.${audio.codec}`),
            value: audio.bitsPerSecond / 1000,
          }),
    );
  }
  return <p className="text-xs text-text-muted">{parts.join(' · ')}</p>;
}

/** The preset menu, with saving the custom settings and removing a saved preset. */
export function PresetPicker({ form }: { readonly form: ExportForm }) {
  const { t } = useTranslation();
  const [naming, setNaming] = useState<string | null>(null);
  const { choice, savedPresets } = form;
  return (
    <div className="flex flex-col gap-1.5">
      <div className="flex items-end gap-2">
        <Field label={t('export.dialog.preset')} className="min-w-0 flex-1">
          <Select
            value={choiceValue(choice)}
            onValueChange={(value) => {
              form.choose(parseChoice(value));
            }}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectGroup>
                {BUILT_IN_PRESETS.map((preset) => (
                  <SelectItem
                    key={preset.id}
                    value={choiceValue({ kind: 'builtIn', id: preset.id })}
                  >
                    {t(`export.presets.${preset.id}`)}
                  </SelectItem>
                ))}
              </SelectGroup>
              {savedPresets.length > 0 && (
                <>
                  <SelectSeparator />
                  <SelectGroup>
                    <SelectLabel>{t('export.dialog.savedPresets')}</SelectLabel>
                    {savedPresets.map((preset) => (
                      <SelectItem
                        key={preset.id}
                        value={choiceValue({ kind: 'saved', id: preset.id })}
                      >
                        {preset.name}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </>
              )}
              <SelectSeparator />
              <SelectItem value="custom">{t('export.presets.custom')}</SelectItem>
            </SelectContent>
          </Select>
        </Field>
        {choice.kind === 'custom' && naming === null && (
          <Button
            type="button"
            variant="secondary"
            onClick={() => {
              setNaming('');
            }}
          >
            {t('export.dialog.savePreset')}
          </Button>
        )}
        {choice.kind === 'saved' && (
          <Button
            type="button"
            variant="ghost"
            onClick={() => {
              void form.removePreset(choice.id);
            }}
          >
            {t('export.dialog.removePreset')}
          </Button>
        )}
      </div>
      {naming !== null && (
        <PresetNameRow
          name={naming}
          onChange={setNaming}
          onCancel={() => {
            setNaming(null);
          }}
          onSave={() => {
            void form.savePreset(naming).then(() => {
              setNaming(null);
            });
          }}
        />
      )}
      <SettingsSummary settings={form.plan.settings} />
    </div>
  );
}

function PresetNameRow({
  name,
  onChange,
  onCancel,
  onSave,
}: {
  readonly name: string;
  readonly onChange: (name: string) => void;
  readonly onCancel: () => void;
  readonly onSave: () => void;
}) {
  const { t } = useTranslation();
  const valid = name.trim() !== '' && name.trim().length <= 60;
  return (
    <div className="flex items-end gap-2">
      <Field label={t('export.dialog.presetName')} className="min-w-0 flex-1">
        <Input
          autoFocus
          value={name}
          maxLength={60}
          onChange={(event) => {
            onChange(event.target.value);
          }}
          onKeyDown={(event) => {
            if (event.key === 'Enter' && valid) {
              event.preventDefault();
              onSave();
            }
          }}
        />
      </Field>
      <Button type="button" variant="ghost" onClick={onCancel}>
        {t('common.cancel')}
      </Button>
      <Button type="button" disabled={!valid} onClick={onSave}>
        {t('common.save')}
      </Button>
    </div>
  );
}

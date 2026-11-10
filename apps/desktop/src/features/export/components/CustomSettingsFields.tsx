import type { ExportContainer, ExportQuality } from '@se/protocol';
import {
  Field,
  Input,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@se/ui';
import { useTranslation } from 'react-i18next';

import {
  AUDIO_BITRATES,
  CONTAINER_VIDEO_CODECS,
  SHORT_SIDES,
  audioCodecsFor,
  isMasterCodec,
  type DraftVideo,
  type ExportDraft,
} from '../services/export-draft';

const CONTAINERS: readonly ExportContainer[] = ['mp4', 'mov', 'webm', 'm4a', 'wav'];
const QUALITIES: readonly ExportQuality[] = ['low', 'medium', 'high', 'veryHigh'];
/** Value of the rate menu meaning « a bitrate typed in Mb/s ». */
const BITRATE = 'bitrate';
/** Value of the sound menu meaning « no sound ». */
const SILENT = 'none';

/** Props of {@link CustomSettingsFields}. */
export interface CustomSettingsFieldsProps {
  readonly draft: ExportDraft;
  readonly onChange: (draft: ExportDraft) => void;
}

/** The fields of custom settings: container, picture codec, size and rate, sound. */
export function CustomSettingsFields({ draft, onChange }: CustomSettingsFieldsProps) {
  const { t } = useTranslation();
  return (
    <div className="grid grid-cols-2 gap-3 rounded-md border border-border p-3">
      <MenuField
        label={t('export.custom.container')}
        value={draft.container}
        options={CONTAINERS.map((container) => [container, t(`export.containers.${container}`)])}
        onChange={(container) => {
          onChange({ ...draft, container: container as ExportContainer });
        }}
      />
      {draft.video !== null && (
        <VideoFields
          container={draft.container}
          video={draft.video}
          onChange={(video) => {
            onChange({ ...draft, video });
          }}
        />
      )}
      <AudioFields draft={draft} onChange={onChange} />
    </div>
  );
}

function VideoFields({
  container,
  video,
  onChange,
}: {
  readonly container: ExportContainer;
  readonly video: DraftVideo;
  readonly onChange: (video: DraftVideo) => void;
}) {
  const { t } = useTranslation();
  const rateValue = video.rate.kind === 'bitrate' ? BITRATE : video.rate.level;
  return (
    <>
      <MenuField
        label={t('export.custom.videoCodec')}
        value={video.codec}
        options={CONTAINER_VIDEO_CODECS[container].map((codec) => [
          codec,
          t(`export.codecs.${codec}`),
        ])}
        onChange={(codec) => {
          onChange({ ...video, codec: codec as DraftVideo['codec'] });
        }}
      />
      <MenuField
        label={t('export.custom.resolution')}
        value={String(video.shortSide)}
        options={SHORT_SIDES.map((side) => [
          String(side),
          side === 'sequence'
            ? t('export.custom.sequenceSize')
            : t('export.custom.shortSide', { value: side }),
        ])}
        onChange={(side) => {
          onChange({ ...video, shortSide: side === 'sequence' ? 'sequence' : Number(side) });
        }}
      />
      {!isMasterCodec(video.codec) && (
        <MenuField
          label={t('export.custom.rate')}
          value={rateValue}
          options={[
            ...QUALITIES.map((level): [string, string] => [level, t(`export.quality.${level}`)]),
            [BITRATE, t('export.custom.bitrate')],
          ]}
          onChange={(value) => {
            onChange({
              ...video,
              rate:
                value === BITRATE
                  ? { kind: 'bitrate', bitsPerSecond: 12_000_000 }
                  : { kind: 'quality', level: value as ExportQuality },
            });
          }}
        />
      )}
      {video.rate.kind === 'bitrate' && !isMasterCodec(video.codec) && (
        <Field label={t('export.custom.megabits')}>
          <Input
            type="number"
            min={0.1}
            max={500}
            step={0.5}
            value={video.rate.bitsPerSecond / 1_000_000}
            onChange={(event) => {
              const megabits = Math.min(500, Math.max(0.1, Number(event.target.value) || 0.1));
              onChange({
                ...video,
                rate: { kind: 'bitrate', bitsPerSecond: Math.round(megabits * 1_000_000) },
              });
            }}
          />
        </Field>
      )}
    </>
  );
}

function AudioFields({ draft, onChange }: CustomSettingsFieldsProps) {
  const { t } = useTranslation();
  const codecs = audioCodecsFor(draft.container, draft.video);
  const { audio } = draft;
  const silentAllowed = draft.video !== null;
  return (
    <>
      <MenuField
        label={t('export.custom.audioCodec')}
        value={audio?.codec ?? SILENT}
        options={[
          ...codecs.map((codec): [string, string] => [codec, t(`export.audioCodecs.${codec}`)]),
          ...(silentAllowed ? [[SILENT, t('export.custom.silent')] as [string, string]] : []),
        ]}
        onChange={(value) => {
          onChange({
            ...draft,
            audio:
              value === SILENT
                ? null
                : {
                    codec: value as (typeof codecs)[number],
                    bitsPerSecond: audio?.bitsPerSecond || 192_000,
                  },
          });
        }}
      />
      {audio !== null && audio.codec !== 'pcm' && (
        <MenuField
          label={t('export.custom.audioBitrate')}
          value={String(audio.bitsPerSecond)}
          options={AUDIO_BITRATES.map((rate) => [
            String(rate),
            t('export.custom.kilobits', { value: rate / 1000 }),
          ])}
          onChange={(value) => {
            onChange({ ...draft, audio: { ...audio, bitsPerSecond: Number(value) } });
          }}
        />
      )}
    </>
  );
}

function MenuField({
  label,
  value,
  options,
  onChange,
}: {
  readonly label: string;
  readonly value: string;
  readonly options: readonly (readonly [string, string])[];
  readonly onChange: (value: string) => void;
}) {
  return (
    <Field label={label}>
      <Select value={value} onValueChange={onChange}>
        <SelectTrigger size="sm">
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {options.map(([optionValue, optionLabel]) => (
            <SelectItem key={optionValue} value={optionValue}>
              {optionLabel}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </Field>
  );
}

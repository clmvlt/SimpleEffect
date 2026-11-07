import type { FadeShape, Flicks } from '@se/protocol';
import {
  PropertyGroup,
  PropertyRow,
  ScrubInput,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { FLICKS_PER_SECOND } from '../../../shared/time';
import { formatVolume, shiftVolume } from '../../timeline';
import { FADE_SHAPES, type AudioClipEditor } from '../hooks/use-audio-clip';

/** Props of {@link AudioSection}. */
export interface AudioSectionProps {
  readonly editor: AudioClipEditor;
}

/** Gain field limits, in decibels (the domain allows −96 to +24). */
const GAIN_RANGE = { min: -60, max: 24 } as const;

function seconds(flicks: number): number {
  return Math.round((flicks / FLICKS_PER_SECOND) * 100) / 100;
}

function flicks(value: number): Flicks {
  return Math.round(value * FLICKS_PER_SECOND) as Flicks;
}

function formatSeconds(value: number): string {
  return `${value.toFixed(2).replace('.', ',')} s`;
}

function isShape(value: string): value is FadeShape {
  return (FADE_SHAPES as readonly string[]).includes(value);
}

/**
 * The « Audio » section of the inspector for one audio clip: its gain (a keyframed volume moves
 * as a whole), its fades and their shapes. Each field sends one command per gesture.
 */
export function AudioSection({ editor }: AudioSectionProps) {
  const { t } = useTranslation();
  const { clip, item } = editor;
  const volume = clip.volumeDb;
  const gain = volume.kind === 'static' ? volume.value : (volume.keyframes[0]?.value ?? 0);
  const [draft, setDraft] = useState<Record<string, number>>({});
  const live = (field: string, value: number) => {
    setDraft((current) => ({ ...current, [field]: value }));
  };
  const settle = (field: string) => {
    setDraft((current) =>
      Object.fromEntries(Object.entries(current).filter(([key]) => key !== field)),
    );
  };
  const duration = seconds(item.timelineRange.duration);
  const fadeField = (field: 'fadeIn' | 'fadeOut', other: number) => (
    <ScrubInput
      id={`inspector-${field}`}
      value={draft[field] ?? seconds(clip[field])}
      min={0}
      max={Math.max(0, duration - other)}
      step={0.05}
      format={formatSeconds}
      onChange={(value) => {
        live(field, value);
      }}
      onCommit={(value) => {
        settle(field);
        editor.updateFades({ [field]: flicks(value) });
      }}
    />
  );
  const shapeField = (field: 'fadeInShape' | 'fadeOutShape') => (
    <Select
      value={clip[field]}
      onValueChange={(value) => {
        if (isShape(value)) editor.updateFades({ [field]: value });
      }}
    >
      <SelectTrigger id={`inspector-${field}`} size="xs" className="w-full">
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {FADE_SHAPES.map((shape) => (
          <SelectItem key={shape} value={shape}>
            {t(`editor.inspector.audio.shapes.${shape}`)}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
  );
  return (
    <PropertyGroup title={t('editor.inspector.audio.title')}>
      <PropertyRow label={t('editor.inspector.audio.gain')} htmlFor="inspector-gain">
        <ScrubInput
          id="inspector-gain"
          value={draft['gain'] ?? gain}
          min={GAIN_RANGE.min}
          max={GAIN_RANGE.max}
          step={0.5}
          format={formatVolume}
          onChange={(value) => {
            live('gain', value);
          }}
          onCommit={(value) => {
            settle('gain');
            editor.setVolume(
              volume.kind === 'static'
                ? { kind: 'static', value }
                : shiftVolume(volume, value - gain),
            );
          }}
        />
      </PropertyRow>
      <PropertyRow label={t('editor.inspector.audio.fadeIn')} htmlFor="inspector-fadeIn">
        {fadeField('fadeIn', seconds(clip.fadeOut))}
      </PropertyRow>
      <PropertyRow label={t('editor.inspector.audio.fadeInShape')} htmlFor="inspector-fadeInShape">
        {shapeField('fadeInShape')}
      </PropertyRow>
      <PropertyRow label={t('editor.inspector.audio.fadeOut')} htmlFor="inspector-fadeOut">
        {fadeField('fadeOut', seconds(clip.fadeIn))}
      </PropertyRow>
      <PropertyRow
        label={t('editor.inspector.audio.fadeOutShape')}
        htmlFor="inspector-fadeOutShape"
      >
        {shapeField('fadeOutShape')}
      </PropertyRow>
    </PropertyGroup>
  );
}

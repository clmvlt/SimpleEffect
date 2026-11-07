import type { TrackEntity, TrackPatch } from '@se/protocol';
import { ScrubInput } from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { formatVolume } from '../services/volume-line';

/** Props of {@link TrackAudioControls}. */
export interface TrackAudioControlsProps {
  readonly track: TrackEntity;
  readonly onChange: (patch: TrackPatch) => void;
}

/** Quietest and loudest track volume offered by the field, in decibels. */
const TRACK_VOLUME_RANGE = { min: -60, max: 12 } as const;

/** A balance for display: `C` in the middle, `G 30` / `D 30` to one side. */
function formatPan(pan: number, left: string, right: string, centre: string): string {
  const percent = Math.round(pan);
  if (percent === 0) return centre;
  return `${percent < 0 ? left : right} ${String(Math.abs(percent))}`;
}

/** Reads `G 30`, `D 30`, `C` or a signed number back as a balance in percent. */
function parsePan(text: string, left: string, right: string): number | null {
  const trimmed = text.trim().toUpperCase();
  if (trimmed === '') return null;
  let sign: number | null = null;
  if (trimmed.startsWith(left.toUpperCase())) sign = -1;
  else if (trimmed.startsWith(right.toUpperCase())) sign = 1;
  const rest = trimmed.replace(/^\p{L}+\s*/u, '').replace(',', '.');
  if (rest === '') return sign === null ? 0 : null;
  const value = Number(rest);
  if (!Number.isFinite(value)) return null;
  return sign === null ? value : sign * Math.abs(value);
}

/**
 * The volume (dB) and stereo balance fields of an audio track header. Dragging shows the value at
 * once; the change is sent once per gesture (one undo step).
 */
export function TrackAudioControls({ track, onChange }: TrackAudioControlsProps) {
  const { t } = useTranslation();
  const [volume, setVolume] = useState<number | null>(null);
  const [pan, setPan] = useState<number | null>(null);
  const sides = {
    left: t('editor.timeline.track.panLeft'),
    right: t('editor.timeline.track.panRight'),
    centre: t('editor.timeline.track.panCentre'),
  };
  return (
    <div className="absolute inset-x-1 bottom-1.5 flex gap-1">
      <ScrubInput
        size="xs"
        className="min-w-0 flex-1"
        label={t('editor.timeline.track.volume')}
        value={volume ?? track.volumeDb}
        min={TRACK_VOLUME_RANGE.min}
        max={TRACK_VOLUME_RANGE.max}
        step={0.5}
        format={formatVolume}
        onChange={setVolume}
        onCommit={(value) => {
          setVolume(null);
          if (value !== track.volumeDb) onChange({ volumeDb: value });
        }}
      />
      <ScrubInput
        size="xs"
        className="w-16 shrink-0"
        label={t('editor.timeline.track.pan')}
        value={pan ?? Math.round(track.pan * 100)}
        min={-100}
        max={100}
        step={1}
        format={(value) => formatPan(value, sides.left, sides.right, sides.centre)}
        parse={(text) => parsePan(text, sides.left, sides.right)}
        onChange={setPan}
        onCommit={(value) => {
          setPan(null);
          if (value / 100 !== track.pan) onChange({ pan: value / 100 });
        }}
      />
    </div>
  );
}

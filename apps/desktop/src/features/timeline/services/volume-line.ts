import type { Animatable, Flicks, Keyframe } from '@se/protocol';
import { interpolateNumber, valueAt } from '@se/engine';

/** Height of a clip's label bar plus its borders: the volume line and the waveform lie below. */
export const LABEL_BAR_HEIGHT = 18;

/** Rows lower than this show no volume line (too little room to aim at it). */
export const MIN_VOLUME_BAND_HEIGHT = 16;

/** The band of a clip of a row `rowHeight` high where the volume line moves (top, height). */
export function volumeBand(rowHeight: number): { readonly top: number; readonly height: number } {
  return { top: LABEL_BAR_HEIGHT, height: Math.max(0, rowHeight - LABEL_BAR_HEIGHT - 2) };
}

/** Loudest volume the line reaches (top of the clip). */
export const VOLUME_TOP_DB = 6;
/** Volume at the bottom of the clip; dragging lower means silence. */
export const VOLUME_BOTTOM_DB = -60;
/** The domain's quietest volume (`MediaClip::MIN_VOLUME_DB`), heard as silence. */
export const SILENCE_DB = -96;
/** The domain's loudest volume (`MediaClip::MAX_VOLUME_DB`). */
export const MAX_VOLUME_DB = 24;

/**
 * Height fraction of a volume in a clip (0 bottom, 1 top): linear in decibels from −60 dB to
 * +6 dB, so 0 dB sits near the top like in other editors; quieter volumes lie on the bottom.
 */
export function volumeToFraction(db: number): number {
  const fraction = (db - VOLUME_BOTTOM_DB) / (VOLUME_TOP_DB - VOLUME_BOTTOM_DB);
  return Math.min(1, Math.max(0, fraction));
}

/** The volume at a height fraction (the bottom edge itself is silence). */
export function fractionToVolume(fraction: number): number {
  if (fraction <= 0) return SILENCE_DB;
  const db = VOLUME_BOTTOM_DB + Math.min(1, fraction) * (VOLUME_TOP_DB - VOLUME_BOTTOM_DB);
  return Math.round(db * 10) / 10;
}

/** A volume kept within the domain's range. */
export function clampVolume(db: number): number {
  return Math.min(MAX_VOLUME_DB, Math.max(SILENCE_DB, db));
}

/** The volume at item-relative `time` (decibels interpolated between keyframes). */
export function volumeAt(volume: Animatable<number>, time: Flicks): number {
  return valueAt(volume, time, interpolateNumber, 0);
}

/** A point of the drawn line: item-relative time and volume. */
export interface VolumePoint {
  readonly time: Flicks;
  readonly db: number;
}

/**
 * The corners of the volume line over an item of `duration`: its ends and its keyframes (the
 * value holds before the first and after the last). Eased segments are sampled, so the line
 * follows what is heard.
 */
export function volumeLinePoints(volume: Animatable<number>, duration: Flicks): VolumePoint[] {
  if (volume.kind === 'static') {
    return [
      { time: 0 as Flicks, db: volume.value },
      { time: duration, db: volume.value },
    ];
  }
  const times = new Set<number>([0, duration]);
  volume.keyframes.forEach((keyframe, index) => {
    times.add(keyframe.time);
    const next = volume.keyframes[index + 1];
    if (next !== undefined && keyframe.easing.kind !== 'linear') {
      for (let step = 1; step < 8; step++) {
        times.add(Math.round(keyframe.time + ((next.time - keyframe.time) * step) / 8));
      }
    }
  });
  return [...times]
    .filter((time) => time >= 0 && time <= duration)
    .sort((left, right) => left - right)
    .map((time) => ({ time: time as Flicks, db: volumeAt(volume, time as Flicks) }));
}

/** Every keyframe (or the value of a static volume) moved by `deltaDb`, kept in range. */
export function shiftVolume(volume: Animatable<number>, deltaDb: number): Animatable<number> {
  if (volume.kind === 'static')
    return { kind: 'static', value: clampVolume(volume.value + deltaDb) };
  return {
    kind: 'keyframed',
    keyframes: volume.keyframes.map((keyframe) => ({
      ...keyframe,
      value: clampVolume(keyframe.value + deltaDb),
    })),
  };
}

const LINEAR = { kind: 'linear' } as const;

/**
 * `volume` with a keyframe at `time` holding what the line reads there (a static volume becomes
 * keyframed); an existing keyframe at that time is kept.
 */
export function addVolumeKeyframe(volume: Animatable<number>, time: Flicks): Animatable<number> {
  const db = volumeAt(volume, time);
  const keyframes = volume.kind === 'static' ? [] : [...volume.keyframes];
  if (keyframes.some((keyframe) => keyframe.time === time)) return volume;
  keyframes.push({ time, value: db, easing: LINEAR });
  keyframes.sort((left, right) => left.time - right.time);
  return { kind: 'keyframed', keyframes };
}

/**
 * `volume` with keyframe `index` moved to `time` (kept strictly between its neighbours, inside
 * the item) at `db`.
 */
export function moveVolumeKeyframe(
  volume: Animatable<number>,
  index: number,
  time: Flicks,
  db: number,
  duration: Flicks,
): Animatable<number> {
  if (volume.kind !== 'keyframed') return volume;
  const previous = volume.keyframes[index - 1];
  const next = volume.keyframes[index + 1];
  const earliest = previous === undefined ? 0 : previous.time + 1;
  const latest = next === undefined ? duration : next.time - 1;
  const clampedTime = Math.min(latest, Math.max(earliest, time)) as Flicks;
  const keyframes = volume.keyframes.map((keyframe, position): Keyframe<number> =>
    position === index ? { ...keyframe, time: clampedTime, value: clampVolume(db) } : keyframe,
  );
  return { kind: 'keyframed', keyframes };
}

/** `volume` without keyframe `index` (the last one leaves a static volume at its value). */
export function removeVolumeKeyframe(
  volume: Animatable<number>,
  index: number,
): Animatable<number> {
  if (volume.kind !== 'keyframed') return volume;
  const removed = volume.keyframes[index];
  const keyframes = volume.keyframes.filter((_, position) => position !== index);
  if (keyframes.length === 0) return { kind: 'static', value: removed?.value ?? 0 };
  return { kind: 'keyframed', keyframes };
}

/** A volume for display: `−∞` at silence, one decimal otherwise. */
export function formatVolume(db: number): string {
  if (db <= SILENCE_DB) return '−∞ dB';
  const rounded = Math.round(db * 10) / 10;
  const text = rounded.toFixed(1).replace('-', '−').replace('.', ',');
  return `${rounded > 0 ? '+' : ''}${text} dB`;
}

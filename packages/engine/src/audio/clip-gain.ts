import type { Flicks } from '@se/protocol';

import { constantPowerWeights } from '../animation/fade-curve';
import { interpolateNumber, valueAt } from '../animation/interpolate';
import { fadeFactor } from '../evaluate/timing';
import { FLICKS_PER_SECOND } from '../time/flicks';
import type { AudioClip, TimeSpan } from './audio-clips';
import { dbToGain } from './mix';

/**
 * Spacing of the points of a varying gain envelope: Web Audio ramps linearly between them, so a
 * volume curve interpolated in decibels, a curved fade or a constant-power cross-fade is followed
 * within a fraction of a decibel.
 */
export const GAIN_STEP_SECONDS = 0.01;

/** A point of a gain envelope: the linear gain `at` seconds after the voice starts. */
export interface GainPoint {
  readonly at: number;
  readonly gain: number;
}

/**
 * Weight of a cross-fade side at `time`: silent before an incoming side and after an outgoing
 * one, full on the other side, constant power in between.
 */
function crossfadeWeight(span: TimeSpan | null, time: number, side: 'in' | 'out'): number {
  if (span === null) return 1;
  if (time < span.start) return side === 'in' ? 0 : 1;
  if (time >= span.end) return side === 'in' ? 1 : 0;
  const weights = constantPowerWeights((time - span.start) / (span.end - span.start));
  return side === 'in' ? weights.incoming : weights.outgoing;
}

/**
 * The linear gain of `clip` at sequence `time` (pure): its volume (decibels interpolated between
 * keyframes, like every animatable property), times its fades along their shapes, times the
 * constant-power weight of the cross-fades it takes part in. Volume and fades beyond the item's
 * edges (played through a cross-fade) take their value at the nearest edge.
 */
export function clipGainAt(clip: AudioClip, time: number): number {
  const duration = (clip.end - clip.start) as Flicks;
  const local = Math.min(duration, Math.max(0, Math.round(time - clip.start))) as Flicks;
  const volume = dbToGain(valueAt(clip.volumeDb, local, interpolateNumber, 0));
  return (
    volume *
    fadeFactor(clip, duration, time - clip.start) *
    crossfadeWeight(clip.crossfadeIn, time, 'in') *
    crossfadeWeight(clip.crossfadeOut, time, 'out')
  );
}

/** Whether `[from, to)` crosses a span (closed at its end, where the level settles). */
function crosses(span: TimeSpan | null, from: number, to: number): boolean {
  return span !== null && span.start < to && span.end >= from;
}

/** Whether the gain of `clip` may change between sequence times `from` and `to`. */
function varies(clip: AudioClip, from: number, to: number): boolean {
  if (clip.volumeDb.kind === 'keyframed') return true;
  const fadeIn = { start: clip.start, end: clip.start + clip.fadeIn };
  const fadeOut = { start: clip.end - clip.fadeOut, end: clip.end };
  return (
    (clip.fadeIn > 0 && crosses(fadeIn, from, to)) ||
    (clip.fadeOut > 0 && crosses(fadeOut, from, to)) ||
    crosses(clip.crossfadeIn, from, to) ||
    crosses(clip.crossfadeOut, from, to)
  );
}

/**
 * The gain envelope of the sound of `clip` between sequence times `from` and `to` (flicks), as
 * heard at normal speed: one point when the gain holds, else points every
 * {@link GAIN_STEP_SECONDS} and at the end, to ramp linearly between (pure: the Web Audio graph
 * only plays it).
 */
export function gainEnvelope(clip: AudioClip, from: number, to: number): GainPoint[] {
  const first = { at: 0, gain: clipGainAt(clip, from) };
  if (to <= from || !varies(clip, from, to)) return [first];
  const length = (to - from) / FLICKS_PER_SECOND;
  const steps = Math.max(1, Math.ceil(length / GAIN_STEP_SECONDS - 1e-9));
  const points: GainPoint[] = [first];
  for (let step = 1; step <= steps; step++) {
    const at = Math.min(length, step * GAIN_STEP_SECONDS);
    points.push({ at, gain: clipGainAt(clip, from + at * FLICKS_PER_SECOND) });
  }
  return points;
}

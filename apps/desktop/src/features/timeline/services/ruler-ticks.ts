import type { Flicks, FrameRate } from '@se/protocol';

import { formatTimecode, frameDuration, nominalFps } from '../../../shared/time';
import { framePixels, timeToX, type Viewport } from './viewport';

/** The graduations of the ruler in view. */
export interface RulerTicks {
  /** Labelled graduations: x in the lanes and timecode. */
  readonly major: readonly { readonly x: number; readonly label: string }[];
  /** Small graduations between labels (x only). */
  readonly minor: readonly number[];
}

/** Minimum pixels between two labels (a timecode is about 70 px wide). */
const MIN_LABEL_SPACING = 96;

/** Minimum pixels between two small graduations. */
const MIN_MINOR_SPACING = 8;

/** Subdivisions tried for small graduations, preferring the finest that keeps them apart. */
const SUBDIVISIONS = [10, 5, 4, 2];

/**
 * Label steps in frames at the nominal rate, from one frame to a day: frame counts, then timecode
 * seconds, minutes and hours (so labels land on round timecodes, also at 29.97).
 */
function labelSteps(rate: FrameRate): number[] {
  const second = nominalFps(rate);
  const frames = [1, 2, 5, 10].filter((count) => count < second);
  const seconds = [1, 2, 5, 10, 15, 30].map((count) => count * second);
  const minutes = [1, 2, 5, 10, 15, 30].map((count) => count * 60 * second);
  const hours = [1, 2, 4, 6, 12, 24].map((count) => count * 3600 * second);
  return [...frames, ...seconds, ...minutes, ...hours];
}

/** The label step (in frames) whose labels are at least {@link MIN_LABEL_SPACING} apart. */
export function majorStep(viewport: Viewport, rate: FrameRate): number {
  const pixels = framePixels(viewport, rate);
  const steps = labelSteps(rate);
  return steps.find((step) => step * pixels >= MIN_LABEL_SPACING) ?? steps.at(-1) ?? 1;
}

/**
 * Graduations of the ruler across the viewport: labels on round timecodes chosen for the zoom,
 * small graduations between them when they fit.
 */
export function rulerTicks(viewport: Viewport, rate: FrameRate): RulerTicks {
  const duration = frameDuration(rate);
  const step = majorStep(viewport, rate);
  const pixels = framePixels(viewport, rate);
  const subdivision = SUBDIVISIONS.find(
    (count) => step % count === 0 && (step / count) * pixels >= MIN_MINOR_SPACING,
  );
  const firstFrame = Math.floor(viewport.scrollTime / duration / step) * step;
  const major: { x: number; label: string }[] = [];
  const minor: number[] = [];
  for (let frame = firstFrame; ; frame += step) {
    const time = (frame * duration) as Flicks;
    const x = timeToX(time, viewport);
    if (x > viewport.width) break;
    major.push({ x, label: formatTimecode(time, rate) });
    if (subdivision === undefined) continue;
    const minorStep = step / subdivision;
    for (let index = 1; index < subdivision; index += 1) {
      minor.push(timeToX(((frame + index * minorStep) * duration) as Flicks, viewport));
    }
  }
  return { major, minor };
}

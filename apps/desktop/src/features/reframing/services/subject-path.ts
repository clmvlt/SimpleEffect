import type { Animatable, FaceBox, Flicks, SubjectDetection, Vec2 } from '@se/protocol';

import { interpolatorOf, valueAtTime } from '../../../shared/animation';

/**
 * The faces of the sample nearest to source time `time` (within one sample interval), most
 * confident first; none when no sample is that close.
 */
export function facesAt(detection: SubjectDetection, time: Flicks): readonly FaceBox[] {
  const { frames } = detection;
  let low = 0;
  let high = frames.length;
  while (low < high) {
    const middle = (low + high) >> 1;
    const frame = frames[middle];
    if (frame !== undefined && frame.time < time) low = middle + 1;
    else high = middle;
  }
  const candidates = [frames[low - 1], frames[low]].filter((frame) => frame !== undefined);
  const nearest = candidates.reduce<(typeof candidates)[number] | undefined>(
    (best, frame) =>
      best === undefined || Math.abs(frame.time - time) < Math.abs(best.time - time) ? frame : best,
    undefined,
  );
  if (nearest === undefined || Math.abs(nearest.time - time) > detection.interval) return [];
  return nearest.faces;
}

/**
 * The focus of a clip at `steps + 1` evenly spaced instants of its `duration` (both ends
 * included), as the engine interpolates it: the trajectory drawn in the viewer.
 */
export function focusTrajectory(
  focus: Animatable<Vec2>,
  duration: Flicks,
  steps: number,
): readonly Vec2[] {
  const interpolate = interpolatorOf('reframeFocus');
  const count = Math.max(1, Math.floor(steps));
  return Array.from({ length: count + 1 }, (_, index) =>
    valueAtTime(focus, Math.round((duration * index) / count) as Flicks, interpolate),
  );
}

/** The focus values of the keyframes of `focus` (none when static). */
export function focusKeyframes(focus: Animatable<Vec2>): readonly Vec2[] {
  return focus.kind === 'keyframed' ? focus.keyframes.map((keyframe) => keyframe.value) : [];
}

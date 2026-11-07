import type { Flicks, SequenceEntity } from '@se/protocol';

import { clampFlicks, frameDuration, snapToFrame } from '../../../shared/time';
import { nextEditPoint, previousEditPoint } from '../services/edit-points';
import { playheadStore, playheadTimeOf, setPlayheadTime } from '../store/playhead-store';

/** Moves the playhead of a sequence, always onto the start of a frame. */
export interface PlayheadControls {
  /** The playhead time now (read when needed, without subscribing). */
  readonly time: () => Flicks;
  /** Moves to the frame holding `time` and scrolls it into view when `reveal` is set. */
  readonly moveTo: (time: Flicks, reveal?: boolean) => void;
  /** Moves by `frames` (negative = back). */
  readonly step: (frames: number) => void;
  readonly goToStart: () => void;
  readonly goToEnd: () => void;
  readonly previousEditPoint: () => void;
  readonly nextEditPoint: () => void;
}

/** Playhead moves for `sequence`; `reveal` scrolls the timeline to a time. */
export function usePlayheadControls(
  sequence: SequenceEntity,
  contentEnd: Flicks,
  points: readonly Flicks[],
  reveal: (time: Flicks) => void,
): PlayheadControls {
  const rate = sequence.format.frameRate;
  const time = () => playheadTimeOf(playheadStore.getState(), sequence.id);
  const moveTo = (target: Flicks, shouldReveal = false) => {
    const snapped = snapToFrame(clampFlicks(Math.max(0, target)), rate, 'floor');
    setPlayheadTime(sequence.id, snapped);
    if (shouldReveal) reveal(snapped);
  };
  return {
    time,
    moveTo,
    step: (frames) => {
      moveTo(clampFlicks(time() + frames * frameDuration(rate)), true);
    },
    goToStart: () => {
      moveTo(0 as Flicks, true);
    },
    goToEnd: () => {
      moveTo(contentEnd, true);
    },
    previousEditPoint: () => {
      const point = previousEditPoint(points, time());
      if (point !== undefined) moveTo(point, true);
    },
    nextEditPoint: () => {
      const point = nextEditPoint(points, time());
      if (point !== undefined) moveTo(point, true);
    },
  };
}

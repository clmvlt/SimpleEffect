/** Forward shuttle speeds reached by pressing L repeatedly (J gives their negatives). */
export const SHUTTLE_SPEEDS: readonly number[] = [1, 2, 4];

/** Which shuttle key was pressed: L plays forward, J plays in reverse. */
export type ShuttleDirection = 'forward' | 'reverse';

/**
 * The playback rate after pressing J or L at `rate` (JKL editing): the first press plays at 1× in
 * its direction, each further press in the same direction doubles up to 4×, and pressing the other
 * direction starts again at 1× that way. K (pause) is rate 0.
 */
export function nextShuttleRate(rate: number, direction: ShuttleDirection): number {
  const sign = direction === 'forward' ? 1 : -1;
  if (rate * sign <= 0) return sign;
  const speed = Math.abs(rate);
  const next = SHUTTLE_SPEEDS.find((candidate) => candidate > speed);
  return sign * (next ?? speed);
}

import type { Flicks } from '@se/protocol';

/**
 * The in and out points of a sequence (the scope of AI requests from prompt 20): `inPoint` is the
 * first instant included, `outPoint` the end (exclusive). Either may be unset.
 */
export interface InOutPoints {
  readonly inPoint: Flicks | null;
  readonly outPoint: Flicks | null;
}

/** No in or out point. */
export const NO_IN_OUT: InOutPoints = { inPoint: null, outPoint: null };

/** A half-open span of a sequence. */
export interface TimeSpan {
  readonly start: Flicks;
  readonly end: Flicks;
}

/** The points with the in point at `time`; an out point at or before it is dropped. */
export function withInPoint(points: InOutPoints, time: Flicks): InOutPoints {
  const outPoint = points.outPoint !== null && points.outPoint <= time ? null : points.outPoint;
  return { inPoint: time, outPoint };
}

/** The points with the out point at `time`; an in point at or after it is dropped. */
export function withOutPoint(points: InOutPoints, time: Flicks): InOutPoints {
  const inPoint = points.inPoint !== null && points.inPoint >= time ? null : points.inPoint;
  return { inPoint, outPoint: time };
}

/** The points spanning two instants in any order (a Maj+drag on the ruler); none if empty. */
export function pointsBetween(a: Flicks, b: Flicks): InOutPoints {
  if (a === b) return NO_IN_OUT;
  return { inPoint: Math.min(a, b) as Flicks, outPoint: Math.max(a, b) as Flicks };
}

/**
 * The span the points mark: from the in point (or the start) to the out point (or `contentEnd`);
 * `null` when no point is set or the span is empty.
 */
export function inOutSpan(points: InOutPoints, contentEnd: Flicks): TimeSpan | null {
  if (points.inPoint === null && points.outPoint === null) return null;
  const start = points.inPoint ?? (0 as Flicks);
  const end = points.outPoint ?? contentEnd;
  return end > start ? { start, end } : null;
}

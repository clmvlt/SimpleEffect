import type { Flicks } from '@se/protocol';

import { FLICKS_PER_SECOND } from '../time/flicks';

/**
 * Where playback stands: sequence time `anchorTime` was at clock time `anchorClock` (seconds on
 * the playback clock, the audio clock in the app) and moves at `rate` (1 = real time, −1 =
 * reverse, 0 = paused). Positions are always derived from the anchor and the clock, never
 * accumulated tick by tick, so playback cannot drift from the clock.
 */
export interface Transport {
  readonly rate: number;
  /** Sequence time in flicks, not rounded (a position caught mid-frame keeps its precision). */
  readonly anchorTime: number;
  readonly anchorClock: number;
}

/** A span of sequence time `[start, end)`: the loop range, or the whole content. */
export interface PlaybackSpan {
  readonly start: Flicks;
  readonly end: Flicks;
}

/** How playback treats the edges of its span. */
export interface PlaybackBounds {
  readonly span: PlaybackSpan;
  /** Wrap around inside the span; otherwise stop at its edge. */
  readonly loop: boolean;
}

/** The sequence time at a clock time; `ended` once a non-looping playback reached an edge. */
export interface TransportPosition {
  /** Sequence time in flicks (not rounded: sub-flick precision keeps audio pieces contiguous). */
  readonly time: number;
  readonly ended: boolean;
}

/** One stretch of playback without wrap: clock `[clockStart, clockEnd)` plays `timeStart…timeEnd`. */
export interface PlaybackSegment {
  readonly clockStart: number;
  readonly clockEnd: number;
  readonly timeStart: number;
  readonly timeEnd: number;
}

function wrap(time: number, span: PlaybackSpan): number {
  const length = span.end - span.start;
  if (length <= 0) return span.start;
  const offset = (((time - span.start) % length) + length) % length;
  return span.start + offset;
}

/**
 * The sequence time of `transport` at `clock`. Looping playback wraps inside the span; otherwise
 * it stops at the span's end (forward) or start (reverse) and reports `ended`.
 */
export function transportPosition(
  transport: Transport,
  clock: number,
  bounds: PlaybackBounds,
): TransportPosition {
  const elapsed = Math.max(0, clock - transport.anchorClock);
  const raw = transport.anchorTime + elapsed * transport.rate * FLICKS_PER_SECOND;
  const { span, loop } = bounds;
  if (transport.rate === 0) return { time: transport.anchorTime, ended: false };
  if (loop) return { time: wrap(raw, span), ended: false };
  if (transport.rate > 0 && raw >= span.end) return { time: span.end, ended: true };
  if (transport.rate < 0 && raw <= span.start) return { time: span.start, ended: true };
  return { time: raw, ended: false };
}

/**
 * The stretches of sequence time played during clock `[from, to)`: one per pass between two loop
 * wraps, in clock order; empty when paused, cut short when a non-looping playback ends. Audio
 * scheduling uses it to place sound across loop points exactly.
 */
export function playbackSegments(
  transport: Transport,
  bounds: PlaybackBounds,
  from: number,
  to: number,
): PlaybackSegment[] {
  const { rate } = transport;
  const { span, loop } = bounds;
  if (rate === 0 || to <= from || span.end <= span.start) return [];
  const segments: PlaybackSegment[] = [];
  let clock = Math.max(from, transport.anchorClock);
  let { time, ended } = transportPosition(transport, clock, bounds);
  const speed = Math.abs(rate) * FLICKS_PER_SECOND;
  while (clock < to && !ended) {
    const remaining = rate > 0 ? span.end - time : time - span.start;
    const edgeClock = clock + remaining / speed;
    if (edgeClock > to) {
      // Same computation as the next window's start: consecutive windows join exactly.
      const timeEnd = transportPosition(transport, to, bounds).time;
      segments.push({ clockStart: clock, clockEnd: to, timeStart: time, timeEnd });
      break;
    }
    if (edgeClock > clock) {
      segments.push({
        clockStart: clock,
        clockEnd: edgeClock,
        timeStart: time,
        timeEnd: edge(rate, span),
      });
    }
    ended = !loop;
    clock = edgeClock;
    time = rate > 0 ? span.start : span.end;
  }
  return segments;
}

function edge(rate: number, span: PlaybackSpan): number {
  return rate > 0 ? span.end : span.start;
}

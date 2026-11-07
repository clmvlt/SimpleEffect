/** Media seconds a playback wants decoded ahead, in the playing direction. */
export interface ReadWindow {
  /** Earliest wanted time (forward: the time on screen; reverse: the far end of the window). */
  readonly from: number;
  /** Latest wanted time (forward: the far end of the window; reverse: the time on screen). */
  readonly to: number;
  readonly direction: 1 | -1;
}

/** What a running reader has covered: it started at `start` and has decoded up to `position`. */
export interface ReaderProgress {
  readonly start: number;
  readonly position: number;
}

/** Keep the reader (extending it forward), or start a new one at `from` reading up to `to`. */
export type ReadDecision =
  | { readonly kind: 'keep' }
  | { readonly kind: 'restart'; readonly from: number; readonly to: number };

/**
 * A forward reader further behind the time on screen than this restarts at that time (seeking
 * to its key frame) instead of decoding every frame in between.
 */
export const MAX_READER_LAG_SECONDS = 0.5;

/**
 * Decides how a sequential reader follows a playback window (pure, `docs/architecture.md` §8).
 *
 * Forward: one decoder keeps running ahead of the time on screen; it restarts only when the time
 * jumped before its start or too far past its position. Reverse: decoders only run forward, so
 * the window before the time on screen is read in chunks — a new chunk starts when less than a
 * third of the window is left above the previous chunk's start, and reads up to that start
 * (frames above it are already cached).
 */
export function planRead(existing: ReaderProgress | null, window: ReadWindow): ReadDecision {
  if (window.direction > 0) {
    const keeps =
      existing !== null &&
      existing.start <= window.from &&
      window.from <= existing.position + MAX_READER_LAG_SECONDS;
    return keeps ? { kind: 'keep' } : { kind: 'restart', from: window.from, to: window.to };
  }
  const length = window.to - window.from;
  const covering = existing !== null && existing.start <= window.to;
  if (covering && window.to - existing.start >= length / 3) return { kind: 'keep' };
  const to = covering && existing.start > window.from ? existing.start : window.to;
  return { kind: 'restart', from: window.from, to };
}

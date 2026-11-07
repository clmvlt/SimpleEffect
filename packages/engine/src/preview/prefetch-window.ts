/** Bounds of the prefetch window, in seconds of playback. */
export const PREFETCH_WINDOW = { initial: 0.5, min: 0.25, max: 2 } as const;

/** On-time playback after which the window shrinks one step, in milliseconds. */
const CALM_MS = 3000;

/**
 * How far ahead playback decodes, adapted to the machine (pure; the caller passes the time): it
 * grows by half each time a frame was not ready when its time came, and shrinks back slowly after
 * a few seconds without late frames, so held frames stay few when decoding keeps up.
 */
export class PrefetchWindow {
  private value: number = PREFETCH_WINDOW.initial;
  private calmSince: number | null = null;

  get seconds(): number {
    return this.value;
  }

  /** A frame had to be waited for at `nowMs`. */
  late(nowMs: number): void {
    this.value = Math.min(PREFETCH_WINDOW.max, this.value * 1.5);
    this.calmSince = nowMs;
  }

  /** A frame was ready in time at `nowMs`. */
  onTime(nowMs: number): void {
    this.calmSince ??= nowMs;
    if (nowMs - this.calmSince < CALM_MS) return;
    this.value = Math.max(PREFETCH_WINDOW.min, this.value * 0.8);
    this.calmSince = nowMs;
  }
}

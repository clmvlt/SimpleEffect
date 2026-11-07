/** Longest interval taken for a display refresh (displays run at 30 Hz or more). */
const MAX_PERIOD_SECONDS = 1 / 30;

/**
 * The display's refresh period, measured on the playback clock between display refreshes
 * (smoothed). A frame drawn during a refresh reaches the screen at the next one, so playback
 * chooses the frame for that moment: without this lead the picture would trail the sound by about
 * one refresh. Stalls longer than a refresh are ignored.
 */
export class RefreshPeriod {
  private last: number | null = null;
  private period = 0;

  /** Seconds between two refreshes (0 until measured). */
  get seconds(): number {
    return this.period;
  }

  /** A refresh ran at clock time `now`. */
  tick(now: number): void {
    const last = this.last;
    this.last = now;
    if (last === null) return;
    const interval = now - last;
    if (interval <= 0 || interval > MAX_PERIOD_SECONDS) return;
    this.period = this.period === 0 ? interval : this.period * 0.9 + interval * 0.1;
  }

  /** Playback stopped: the next refresh starts a new interval (the estimate is kept). */
  pause(): void {
    this.last = null;
  }
}

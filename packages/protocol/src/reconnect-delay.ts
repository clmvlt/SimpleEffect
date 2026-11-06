/** Bounds of the exponential reconnection backoff. */
export interface ReconnectDelayOptions {
  /** Delay before the first retry. */
  readonly initialMs: number;
  /** Upper bound of the delay. */
  readonly maxMs: number;
}

/** Defaults: 250 ms, doubling, capped at 5 s (the Electron supervisor restarts a dead core first). */
export const DEFAULT_RECONNECT_DELAY: ReconnectDelayOptions = { initialMs: 250, maxMs: 5_000 };

/** Delay before retry number `failures + 1`: `initialMs × 2^failures`, capped at `maxMs`. */
export function reconnectDelayMs(
  failures: number,
  { initialMs, maxMs }: ReconnectDelayOptions = DEFAULT_RECONNECT_DELAY,
): number {
  return Math.min(maxMs, initialMs * 2 ** Math.max(0, failures));
}

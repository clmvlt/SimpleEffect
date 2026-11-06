/** First restart delay after a crash. */
const INITIAL_DELAY_MS = 250;
/** Upper bound so a crash loop never waits longer than this between attempts. */
const MAX_DELAY_MS = 10_000;

/**
 * Delay before restarting se-core after `failedAttempts` consecutive failures (0-based):
 * 250 ms, 500 ms, 1 s, 2 s… capped at 10 s.
 */
export function restartDelayMs(failedAttempts: number): number {
  const exponent = Math.max(0, Math.floor(failedAttempts));
  return Math.min(INITIAL_DELAY_MS * 2 ** exponent, MAX_DELAY_MS);
}

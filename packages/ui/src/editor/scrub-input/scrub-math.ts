/** Bounds and step of a scrubbable number. */
export interface NumericRange {
  readonly min?: number | undefined;
  readonly max?: number | undefined;
  readonly step: number;
}

/** Modifier keys that change the scrubbing speed. */
export interface ScrubModifiers {
  readonly shiftKey: boolean;
  readonly altKey: boolean;
}

/** Speed multipliers: Shift = fast, Alt = fine. */
export interface ScrubSpeeds {
  readonly fast: number;
  readonly fine: number;
}

/** Keeps `value` inside `[min, max]` (missing bounds are open). */
export function clampToRange(
  value: number,
  { min, max }: Pick<NumericRange, 'min' | 'max'>,
): number {
  return Math.min(
    max ?? Number.POSITIVE_INFINITY,
    Math.max(min ?? Number.NEGATIVE_INFINITY, value),
  );
}

/**
 * Number of decimals needed to write `step` (`0.25` → 2, `5` → 0, `1e-3` → 3), ignoring binary
 * noise beyond 12 significant digits (`0.1 * 0.1` → 2).
 */
export function decimalsOf(step: number): number {
  if (!Number.isFinite(step) || Number.isInteger(step)) {
    return 0;
  }
  const [mantissa = '', exponent] = Number(step.toPrecision(12)).toExponential().split('e');
  const mantissaDecimals = mantissa.split('.')[1]?.length ?? 0;
  return Math.max(0, mantissaDecimals - Number(exponent));
}

/** Rounds to `decimals` places, removing binary noise (`0.1 + 0.2` → `0.3`). */
export function roundToDecimals(value: number, decimals: number): number {
  return Number(value.toFixed(Math.min(decimals, 100)));
}

/** The increment for the held modifiers: `step`, `step × fast` (Shift) or `step × fine` (Alt). */
export function incrementFor(step: number, modifiers: ScrubModifiers, speeds: ScrubSpeeds): number {
  if (modifiers.shiftKey) {
    return step * speeds.fast;
  }
  return modifiers.altKey ? step * speeds.fine : step;
}

/** Snaps `value` to the nearest multiple of `increment`, written without binary noise. */
export function snapToIncrement(value: number, increment: number): number {
  return roundToDecimals(Math.round(value / increment) * increment, decimalsOf(increment));
}

/**
 * Value after a keyboard step: `value ± increment`, clamped, rounded to the finest precision the
 * widget can produce so repeated steps never accumulate float noise.
 */
export function stepValue(
  value: number,
  direction: 1 | -1,
  increment: number,
  range: NumericRange,
  speeds: ScrubSpeeds,
): number {
  const precision = Math.max(decimalsOf(range.step * speeds.fine), decimalsOf(increment));
  return clampToRange(roundToDecimals(value + direction * increment, precision), range);
}

/** Default display: the number with no trailing zeros beyond the step precision (`12.50` → `12.5`). */
export function formatScrubValue(value: number, decimals: number): string {
  return String(roundToDecimals(value, decimals));
}

/**
 * Default parser for typed values: accepts a decimal comma (French keyboards), surrounding spaces
 * and a trailing unit (`12,5 px` → 12.5). Returns `null` when the text is not a finite number.
 */
export function parseScrubValue(text: string): number | null {
  const match = /^\s*([+-]?(?:\d+(?:[.,]\d*)?|[.,]\d+))\s*[^\d\s]*\s*$/.exec(text);
  if (match?.[1] === undefined) {
    return null;
  }
  const parsed = Number(match[1].replace(',', '.'));
  return Number.isFinite(parsed) ? parsed : null;
}

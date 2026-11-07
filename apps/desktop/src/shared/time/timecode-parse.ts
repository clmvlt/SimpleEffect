import type { FrameRate } from '@se/protocol';

import { frameDuration, nominalFps } from './frame-rate';
import { failure, success, type TimeResult } from './time-result';

/**
 * The timecode grammar of `se_domain::parse_timecode` (`timecode/parse.rs`), on BigInt magnitudes
 * so no intermediate value loses precision; the range is checked once at the end.
 */

/** Why a timecode is refused (Rust `TimecodeErrorReason` variants in camelCase). */
export type TimecodeErrorCode =
  | 'empty'
  | 'unrecognised'
  | 'missingUnit'
  | 'minutesOutOfRange'
  | 'secondsOutOfRange'
  | 'framesOutOfRange'
  | 'frameRateRequired'
  | 'tooManyDecimals'
  | 'outOfRange';

type Parsed<T> = TimeResult<T, TimecodeErrorCode>;

const FLICKS_PER_SECOND = 705_600_000n;
const MAX_DECIMALS = 9;
const MAX_SAFE = BigInt(Number.MAX_SAFE_INTEGER);
const DIGITS = /^[0-9]+$/u;

interface Decimal {
  readonly integer: bigint;
  readonly fraction: bigint;
  readonly decimals: number;
}

function parseDigits(text: string): Parsed<bigint> {
  return DIGITS.test(text) ? success(BigInt(text)) : failure('unrecognised');
}

function parseDecimal(text: string): Parsed<Decimal> {
  const dot = text.indexOf('.');
  const integer = parseDigits(dot === -1 ? text : text.slice(0, dot));
  if (!integer.ok) return integer;
  if (dot === -1) return success({ integer: integer.value, fraction: 0n, decimals: 0 });
  const fractionText = text.slice(dot + 1);
  if (!DIGITS.test(fractionText)) return failure('unrecognised');
  if (fractionText.length > MAX_DECIMALS) return failure('tooManyDecimals');
  return success({
    integer: integer.value,
    fraction: BigInt(fractionText),
    decimals: fractionText.length,
  });
}

/** Seconds and decimals to flicks, the fraction rounded to the nearest flick (halves up). */
function decimalToFlicks({ integer, fraction, decimals }: Decimal): bigint {
  const scale = 10n ** BigInt(decimals);
  const rounded = (fraction * FLICKS_PER_SECOND * 2n + scale) / (scale * 2n);
  return integer * FLICKS_PER_SECOND + rounded;
}

function below60(value: bigint, error: TimecodeErrorCode): Parsed<bigint> {
  return value < 60n ? success(value) : failure(error);
}

/** `HH:MM:SS;FF`: one `;`, right before the frames. */
function isDropFrameNotation(body: string, fieldCount: number): boolean {
  return (
    fieldCount === 4 &&
    body.split(';').length === 2 &&
    body.lastIndexOf(';') > body.lastIndexOf(':')
  );
}

function parseMinutesSeconds(minutes: string, seconds: string): Parsed<bigint> {
  // Same checking order as Rust, so both sides report the same error.
  const decimal = parseDecimal(seconds);
  if (!decimal.ok) return decimal;
  const checked = below60(decimal.value.integer, 'secondsOutOfRange');
  if (!checked.ok) return checked;
  const whole = parseDigits(minutes);
  if (!whole.ok) return whole;
  return success(whole.value * 60n * FLICKS_PER_SECOND + decimalToFlicks(decimal.value));
}

function parseHoursMinutesSeconds(hours: string, minutes: string, seconds: string): Parsed<bigint> {
  const parsedMinutes = parseDigits(minutes);
  if (!parsedMinutes.ok) return parsedMinutes;
  const checkedMinutes = below60(parsedMinutes.value, 'minutesOutOfRange');
  if (!checkedMinutes.ok) return checkedMinutes;
  const decimal = parseDecimal(seconds);
  if (!decimal.ok) return decimal;
  const checkedSeconds = below60(decimal.value.integer, 'secondsOutOfRange');
  if (!checkedSeconds.ok) return checkedSeconds;
  const parsedHours = parseDigits(hours);
  if (!parsedHours.ok) return parsedHours;
  const whole = parsedHours.value * 3600n + parsedMinutes.value * 60n;
  return success(whole * FLICKS_PER_SECOND + decimalToFlicks(decimal.value));
}

function parseSmpte(fields: readonly string[], rate: FrameRate | null): Parsed<bigint> {
  const [hours, minutes, seconds, frames] = fields.map(parseDigits);
  if (hours === undefined || minutes === undefined || seconds === undefined) {
    return failure('unrecognised');
  }
  if (!hours.ok) return hours;
  if (!minutes.ok) return minutes;
  if (!seconds.ok) return seconds;
  const checkedMinutes = below60(minutes.value, 'minutesOutOfRange');
  if (!checkedMinutes.ok) return checkedMinutes;
  const checkedSeconds = below60(seconds.value, 'secondsOutOfRange');
  if (!checkedSeconds.ok) return checkedSeconds;
  if (frames === undefined || !frames.ok) return frames ?? failure('unrecognised');
  if (rate === null) return failure('frameRateRequired');
  const nominal = BigInt(nominalFps(rate));
  if (frames.value >= nominal) return failure('framesOutOfRange');
  const wholeSeconds = hours.value * 3600n + minutes.value * 60n + seconds.value;
  return success((wholeSeconds * nominal + frames.value) * BigInt(frameDuration(rate)));
}

function parseClock(body: string, rate: FrameRate | null): Parsed<bigint> {
  const fields = body.split(/[:;]/u);
  if (body.includes(';') && !isDropFrameNotation(body, fields.length)) {
    return failure('unrecognised');
  }
  const [first = '', second = '', third = ''] = fields;
  switch (fields.length) {
    case 2:
      return parseMinutesSeconds(first, second);
    case 3:
      return parseHoursMinutesSeconds(first, second, third);
    case 4:
      return parseSmpte(fields, rate);
    default:
      return failure('unrecognised');
  }
}

/** The signed flicks magnitude of an already trimmed timecode. */
export function parseTrimmed(text: string, rate: FrameRate | null): Parsed<number> {
  if (text === '') return failure('empty');
  const negative = text.startsWith('-');
  const body = negative ? text.slice(1) : text;
  let magnitude: Parsed<bigint>;
  if (body.endsWith('s')) {
    const decimal = parseDecimal(body.slice(0, -1));
    magnitude = decimal.ok ? success(decimalToFlicks(decimal.value)) : decimal;
  } else if (/[:;]/u.test(body)) {
    magnitude = parseClock(body, rate);
  } else {
    return failure(parseDecimal(body).ok ? 'missingUnit' : 'unrecognised');
  }
  if (!magnitude.ok) return magnitude;
  if (magnitude.value > MAX_SAFE) return failure('outOfRange');
  const value = Number(magnitude.value);
  return success(negative && value !== 0 ? -value : value);
}

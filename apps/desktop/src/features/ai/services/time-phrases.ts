import type { Flicks, FrameRate } from '@se/protocol';

import { parseTimecode, secondsToFlicks, snapToFrame } from '../../../shared/time';

/**
 * Time ranges written in French in a prompt (« de 51s à 54s », « entre 1:02 et 1:10 »,
 * « 00:51-00:54 », « à partir de 2 min », « les 10 premières secondes »…), found so the composer
 * can propose them as the task's scope. Pure: the sequence's rate and end come from the caller.
 */

/** What the phrases are read against. */
export interface TimePhraseContext {
  /** The sequence's rate: SMPTE timecodes (`00:00:51:12`) and the snapping of every bound. */
  readonly frameRate: FrameRate;
  /** End of the sequence's content: « à partir de … » and « les … dernières secondes » end there. */
  readonly sequenceEnd: Flicks;
}

/** A time range found in a prompt. */
export interface TimePhrase {
  readonly start: Flicks;
  readonly end: Flicks;
  /** The words it was read from, as written (« de 51s à 54s »). */
  readonly text: string;
  /** Where they start in the prompt. */
  readonly index: number;
}

const NUMBER = String.raw`\d+(?:[.,]\d+)?`;
const UNIT = String.raw`(?:heures?|h|minutes?|min|mn|m|secondes?|sec|s)`;
const SECONDS_UNIT = String.raw`(?:secondes?|sec|s)`;
const CLOCK = String.raw`\d{1,2}(?::\d{2}){1,3}(?:[.,]\d{1,3})?`;
/** A number with a unit, optionally followed by a smaller part (« 2 min 30 », « 1m30s »). */
const WITH_UNIT = String.raw`${NUMBER}\s*${UNIT}(?:\s*\d{1,2}(?:\s*${SECONDS_UNIT})?)?(?!\p{L})`;
/** One instant: a clock, a number with a unit, or a bare number (its unit comes from the other). */
const POINT = String.raw`(${CLOCK}|${WITH_UNIT}|${NUMBER})`;
const TO = String.raw`(?:à|a|jusqu['’]?\s*(?:à|a)|-|–|—|→|->)`;
const ARTICLE = String.raw`(?:(?:la|le|l['’])\s*)?`;

/** Ranges with both bounds written, in the order they are tried at one position. */
const RANGE_PATTERNS: readonly RegExp[] = [
  new RegExp(
    String.raw`(?<!\p{L})(?:de|depuis|du)\s+${ARTICLE}${POINT}\s*${TO}\s*${ARTICLE}${POINT}`,
    'giu',
  ),
  new RegExp(String.raw`(?<!\p{L})entre\s+${ARTICLE}${POINT}\s+et\s+${ARTICLE}${POINT}`, 'giu'),
  new RegExp(String.raw`(?<![\p{L}\d:.,])${POINT}\s*${TO}\s*${POINT}`, 'giu'),
];
const FROM_PATTERN = new RegExp(
  String.raw`(?<!\p{L})(?:à\s+partir\s+de|a\s+partir\s+de|dès|depuis)\s+${ARTICLE}${POINT}`,
  'giu',
);
const UNTIL_PATTERN = new RegExp(
  String.raw`(?<!\p{L})jusqu['’]?\s*(?:à|a)\s+${ARTICLE}${POINT}`,
  'giu',
);
const EDGE_PATTERN = new RegExp(
  String.raw`(?<!\p{L})(?:les|la)\s+(?:(\d+(?:[.,]\d+)?)\s+)?(premi[eè]res?|derni[eè]res?)\s+(?:(\d+(?:[.,]\d+)?)\s+)?(secondes?|minutes?)(?!\p{L})`,
  'giu',
);

/** An instant as written: seconds, and whether a unit or clock said what it means. */
interface Instant {
  readonly seconds: number;
  readonly explicit: boolean;
}

/** Seconds of a unit word. */
function unitSeconds(unit: string): number {
  const word = unit.toLowerCase();
  if (word.startsWith('h')) return 3600;
  if (word.startsWith('m')) return 60;
  return 1;
}

function decimal(text: string): number {
  return Number(text.replace(',', '.'));
}

/** Reads one written instant; `null` when it is not one (minutes over 59, unknown words…). */
function readInstant(raw: string, rate: FrameRate): Instant | null {
  const text = raw.trim();
  if (text.includes(':')) {
    const parsed = parseTimecode(text.replace(',', '.'), rate);
    return parsed.ok ? { seconds: parsed.value / 705_600_000, explicit: true } : null;
  }
  const match = new RegExp(
    String.raw`^(${NUMBER})\s*(${UNIT})?(?:\s*(\d{1,2})(?:\s*${SECONDS_UNIT})?)?$`,
    'iu',
  ).exec(text);
  if (match === null) return null;
  const [, amount = '', unit, part] = match;
  if (unit === undefined) return { seconds: decimal(amount), explicit: false };
  const scale = unitSeconds(unit);
  const smaller = part === undefined ? 0 : Number(part);
  if (part !== undefined && (scale === 1 || smaller >= 60)) return null;
  return { seconds: decimal(amount) * scale + smaller * (scale / 60), explicit: true };
}

/** The unit a bare bound borrows from the other one (« de 51 à 54 s » → seconds). */
function borrowedScale(raw: string): number | null {
  const unit = new RegExp(String.raw`^${NUMBER}\s*(${UNIT})`, 'iu').exec(raw.trim())?.[1];
  return unit === undefined ? null : unitSeconds(unit);
}

/** Both bounds of a written range, a bare one taking the other's unit; `null` when unclear. */
function readBounds(first: string, second: string, rate: FrameRate): [number, number] | null {
  const start = readInstant(first, rate);
  const end = readInstant(second, rate);
  if (start === null || end === null || (!start.explicit && !end.explicit)) return null;
  if (start.explicit && end.explicit) return [start.seconds, end.seconds];
  const scale = borrowedScale(start.explicit ? first : second);
  if (scale === null) return null;
  return start.explicit
    ? [start.seconds, end.seconds * scale]
    : [start.seconds * scale, end.seconds];
}

/** Seconds as flicks on the nearest frame; `null` outside the supported range. */
function toFrame(seconds: number, rate: FrameRate): Flicks | null {
  const flicks = secondsToFlicks(seconds);
  return flicks.ok ? snapToFrame(flicks.value, rate, 'nearest') : null;
}

/** A phrase from bounds in seconds, if they make a non-empty range. */
function phrase(
  match: RegExpExecArray,
  seconds: readonly [number, number] | null,
  context: TimePhraseContext,
): TimePhrase | null {
  if (seconds === null) return null;
  const start = toFrame(seconds[0], context.frameRate);
  const end = toFrame(Math.min(seconds[1], Number.MAX_SAFE_INTEGER), context.frameRate);
  if (start === null || end === null || start < 0 || end <= start) return null;
  return { start, end, text: match[0].trim(), index: match.index };
}

function endSeconds(context: TimePhraseContext): number {
  return context.sequenceEnd / 705_600_000;
}

/** Every phrase one pattern finds. */
function findAll(
  pattern: RegExp,
  read: (match: RegExpExecArray) => readonly [number, number] | null,
  text: string,
  context: TimePhraseContext,
): TimePhrase[] {
  const found: TimePhrase[] = [];
  for (const match of text.matchAll(pattern)) {
    const result = phrase(match, read(match), context);
    if (result !== null) found.push(result);
  }
  return found;
}

function readEdge(match: RegExpExecArray, context: TimePhraseContext): [number, number] | null {
  const [, before, edge = '', after, unit = ''] = match;
  const count = before ?? after;
  const plural = unit.toLowerCase().endsWith('s');
  if ((count === undefined) === plural) return null;
  const length = (count === undefined ? 1 : decimal(count)) * unitSeconds(unit);
  const end = endSeconds(context);
  return edge.toLowerCase().startsWith('prem') ? [0, length] : [Math.max(0, end - length), end];
}

/**
 * The first time range written in `text`, or `null`. At one position the longest reading wins
 * (« de 51s à 54s » over « 51s à 54s »). Bounds snap to the nearest frame; a range must not be
 * empty or reversed. Bare numbers are read only as one bound of a range whose other bound has a
 * unit (« entre 51 et 54 secondes »); three-part clocks are hours, minutes, seconds (`1:02:03`),
 * four-part ones SMPTE timecodes at the sequence's rate.
 */
export function findTimePhrase(text: string, context: TimePhraseContext): TimePhrase | null {
  const { frameRate } = context;
  const bounded = (match: RegExpExecArray) => readBounds(match[1] ?? '', match[2] ?? '', frameRate);
  const explicit = (raw: string | undefined) => {
    const instant = raw === undefined ? null : readInstant(raw, frameRate);
    return instant?.explicit === true ? instant.seconds : null;
  };
  const candidates = [
    ...RANGE_PATTERNS.flatMap((pattern) => findAll(pattern, bounded, text, context)),
    ...findAll(
      FROM_PATTERN,
      (match) => {
        const start = explicit(match[1]);
        return start === null ? null : [start, endSeconds(context)];
      },
      text,
      context,
    ),
    ...findAll(
      UNTIL_PATTERN,
      (match) => {
        const end = explicit(match[1]);
        return end === null ? null : [0, end];
      },
      text,
      context,
    ),
    ...findAll(EDGE_PATTERN, (match) => readEdge(match, context), text, context),
  ];
  return candidates.reduce<TimePhrase | null>((best, candidate) => {
    if (best === null || candidate.index < best.index) return candidate;
    if (candidate.index === best.index && candidate.text.length > best.text.length)
      return candidate;
    return best;
  }, null);
}

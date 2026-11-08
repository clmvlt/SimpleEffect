import { shortTime } from './short-time';

/**
 * Tool calls of an agent as compact lines for the task log (« add_text “Abonne-toi”
 * 00:51→00:54 »), read from the input the provider reported (JSON, possibly cut with « … »).
 * Tool names stay as agents know them (English, like `docs/ai-tools.md`).
 */

type Json = string | number | boolean | null | Json[] | { [key: string]: Json };

const MAX_QUOTE = 40;
const MAX_RAW = 60;
const UUID = /[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/giu;
const CLOCK = /^(?:(\d+):)?(\d{1,2}):(\d{1,2}(?:\.\d+)?)$/u;
const SECONDS = /^(\d+(?:\.\d+)?)\s*s?$/u;

function parse(text: string): Record<string, Json> | null {
  try {
    const value: unknown = JSON.parse(text);
    return typeof value === 'object' && value !== null && !Array.isArray(value)
      ? (value as Record<string, Json>)
      : null;
  } catch {
    return null;
  }
}

function cut(text: string, length: number): string {
  return text.length > length ? text.slice(0, length - 1) + '…' : text;
}

/** A time of a tool input (seconds, `51s` or a timecode) as a short time; `null` otherwise. */
export function inputTime(value: Json | undefined): string | null {
  if (typeof value === 'number' && Number.isFinite(value)) return shortTime(value);
  if (typeof value !== 'string') return null;
  const text = value.trim();
  const seconds = SECONDS.exec(text);
  if (seconds !== null) return shortTime(Number(seconds[1]));
  const clock = CLOCK.exec(text);
  if (clock === null) return null;
  const [, hours = '0', minutes = '0', rest = '0'] = clock;
  return shortTime(Number(hours) * 3600 + Number(minutes) * 60 + Number(rest));
}

function quoteOf(input: Record<string, Json>): string | null {
  for (const key of ['text', 'label', 'name', 'content']) {
    const value = input[key];
    if (typeof value === 'string' && value.trim() !== '') {
      return `“${cut(value.trim(), MAX_QUOTE)}”`;
    }
  }
  return null;
}

function presetOf(input: Record<string, Json>): string | null {
  const preset = input.preset;
  if (typeof preset === 'string') return preset;
  for (const key of ['animation_in', 'animation_out', 'animation']) {
    const animation = input[key];
    if (typeof animation === 'object' && animation !== null && !Array.isArray(animation)) {
      const name = animation.preset;
      if (typeof name === 'string') return name;
    }
  }
  return typeof input.shape === 'string' ? input.shape : null;
}

/** `00:51→00:54` from `start`/`end`, `start`/`duration`, a `range` or a single `at`/`time`. */
function spanOf(input: Record<string, Json>): string | null {
  const range = input.range;
  const source =
    typeof range === 'object' && range !== null && !Array.isArray(range) ? range : input;
  const start = inputTime(source.start);
  const end = inputTime(source.end);
  if (start !== null && end !== null) return `${start}→${end}`;
  const duration = typeof source.duration === 'number' ? source.duration : null;
  const startSeconds = typeof source.start === 'number' ? source.start : null;
  if (startSeconds !== null && duration !== null) {
    return `${shortTime(startSeconds)}→${shortTime(startSeconds + duration)}`;
  }
  if (start !== null) return `${start}→`;
  const at = inputTime(input.at) ?? inputTime(input.time);
  return at === null ? null : `@${at}`;
}

function countOf(input: Record<string, Json>): string | null {
  for (const key of ['item_ids', 'items', 'actions', 'times', 'keyframes']) {
    const value = input[key];
    if (Array.isArray(value)) return `×${String(value.length)}`;
  }
  return null;
}

/** One line for a tool call: its name and the salient parts of its input. */
export function toolCallLine(tool: string, inputSummary: string): string {
  const name = tool === '' ? '?' : tool;
  const input = parse(inputSummary);
  if (input === null) {
    const raw = inputSummary.trim();
    return raw === '' || raw === '{}' ? name : `${name} ${cut(raw, MAX_RAW)}`;
  }
  const parts = [quoteOf(input), presetOf(input), spanOf(input), countOf(input)].filter(
    (part): part is string => part !== null,
  );
  return [name, ...parts].join(' ');
}

/** Ids (UUIDs) named in a call's input and output: the items it read, created or changed. */
export function idsInCall(inputSummary: string, outputSummary: string | null): string[] {
  const text = inputSummary + ' ' + (outputSummary ?? '');
  return [...new Set(Array.from(text.matchAll(UUID), (match) => match[0].toLowerCase()))];
}

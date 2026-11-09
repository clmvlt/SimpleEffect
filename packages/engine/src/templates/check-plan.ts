import type { TemplateParamInput } from './contract';

/** Length assumed when `meta.defaultDuration` is unusable (the core refuses it anyway). */
const FALLBACK_DURATION = 5;

/** What a check renders with: the declared length and default values. */
export interface CheckDefaults {
  /** Seconds. */
  readonly duration: number;
  readonly params: Readonly<Record<string, TemplateParamInput>>;
  /** Font families of `font` parameters' defaults (given to the Worker before rendering). */
  readonly families: readonly string[];
}

/**
 * The length and default values a check renders with, read leniently from the module's exports
 * (`meta` as JSON, `params` as `[name, schema]` pairs): the core reads them strictly afterwards,
 * so a malformed declaration is reported by the core, never guessed here.
 */
export function checkDefaults(metaJson: string, paramsJson: string): CheckDefaults {
  const meta = parse(metaJson);
  const declared = meta === null ? null : meta['defaultDuration'];
  const duration =
    typeof declared === 'number' && declared > 0 && declared <= 3600 ? declared : FALLBACK_DURATION;
  const params: Record<string, TemplateParamInput> = {};
  const families: string[] = [];
  const pairs: unknown = parseAny(paramsJson);
  for (const pair of Array.isArray(pairs) ? pairs : []) {
    if (!Array.isArray(pair) || typeof pair[0] !== 'string') continue;
    const [name, schema] = pair as [string, unknown];
    const value = defaultOf(schema);
    params[name] = value;
    if (isRecord(schema) && schema['format'] === 'font' && typeof value === 'string') {
      families.push(value);
    }
  }
  return { duration, params, families };
}

/** The instants a check renders, in seconds. */
export interface CheckInstants {
  /** Rendered in each pass (start, entrance, middle, exit, end), timed in the second. */
  readonly timed: readonly number[];
  /** Returned as pictures: entrance, middle, exit. */
  readonly pictures: readonly number[];
}

/**
 * The instants of a check for an instance of `duration` seconds: the entrance and exit sit
 * 0.4 s in from the edges (20 % of shorter instances), where most templates are mid-animation.
 */
export function checkInstants(duration: number): CheckInstants {
  const edge = Math.min(0.4, duration * 0.2);
  const round = (time: number) => Math.round(time * 1000) / 1000;
  const entrance = round(edge);
  const middle = round(duration / 2);
  const exit = round(duration - edge);
  const end = round(Math.max(0, duration - 1 / 60));
  const timed = [...new Set([0, entrance, middle, exit, end])];
  return { timed, pictures: [entrance, middle, exit] };
}

function defaultOf(schema: unknown): TemplateParamInput {
  if (!isRecord(schema)) return null;
  const value = schema['default'];
  if (schema['format'] === 'asset' || schema['format'] === 'image') return null;
  if (typeof value === 'string' || typeof value === 'number' || typeof value === 'boolean') {
    return value;
  }
  return null;
}

function parse(json: string): Record<string, unknown> | null {
  const value = parseAny(json);
  return isRecord(value) ? value : null;
}

function parseAny(json: string): unknown {
  try {
    return JSON.parse(json);
  } catch {
    return null;
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

import type { NoticeEntry, ThirdPartyNotices } from '../shared/bridge-api';

/** The `formatVersion` of `third-party-notices.json` this app reads (`scripts/lib/notices.mjs`). */
export const NOTICES_FORMAT_VERSION = 1;

const SOURCES = new Set(['component', 'npm', 'cargo']);

/**
 * Parses `third-party-notices.json`. A file of another format or with a malformed entry is
 * refused with an error naming the problem: it is written by the build, so a mismatch is a bug.
 */
export function parseThirdPartyNotices(json: string): ThirdPartyNotices {
  const value: unknown = JSON.parse(json);
  if (!isRecord(value) || value.formatVersion !== NOTICES_FORMAT_VERSION) {
    throw new TypeError(
      `third-party-notices.json is not of format ${String(NOTICES_FORMAT_VERSION)}`,
    );
  }
  const { entries, texts } = value;
  if (!Array.isArray(entries) || !isRecord(texts)) {
    throw new TypeError('third-party-notices.json needs `entries` and `texts`');
  }
  const textsById: Record<string, string> = {};
  for (const [id, text] of Object.entries(texts)) {
    if (typeof text !== 'string') throw new TypeError(`licence text ${id} is not a string`);
    textsById[id] = text;
  }
  return { entries: entries.map((entry, index) => parseEntry(entry, index)), texts: textsById };
}

function parseEntry(value: unknown, index: number): NoticeEntry {
  if (!isRecord(value)) throw new TypeError(`notice ${String(index)} is not an object`);
  const { source, name, version, license, url, note, textIds } = value;
  const strings = [name, version, license].every((field) => typeof field === 'string');
  const ids = Array.isArray(textIds) && textIds.every((id) => Number.isInteger(id));
  if (typeof source !== 'string' || !SOURCES.has(source) || !strings || !ids) {
    throw new TypeError(`notice ${String(index)} is malformed`);
  }
  return {
    source: source as NoticeEntry['source'],
    name: name as string,
    version: version as string,
    license: license as string,
    ...(typeof url === 'string' ? { url } : {}),
    ...(typeof note === 'string' ? { note } : {}),
    textIds: textIds as number[],
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value);
}

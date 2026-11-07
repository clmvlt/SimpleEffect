import type { Flicks, FrameRate } from '@se/protocol';

import { flicksToFrames } from './flicks';
import { nominalFps } from './frame-rate';
import type { TimeResult } from './time-result';
import { parseTrimmed, type TimecodeErrorCode } from './timecode-parse';

export type { TimecodeErrorCode } from './timecode-parse';

/**
 * Parses a time typed by a user or an agent, like `se_domain::parse_timecode` (checked against
 * `docs/test-vectors/timecode.json`): `51s`, `51.5s` (≤ 9 decimals), `00:51`, `1:02.500`,
 * `1:00:00`, `00:00:51:12` (SMPTE, needs `rate`; `;` before the frames is read as non-drop) and a
 * leading `-`. A bare number is refused as ambiguous (`missingUnit`).
 */
export function parseTimecode(
  text: string,
  rate: FrameRate | null,
): TimeResult<Flicks, TimecodeErrorCode> {
  return parseTrimmed(text.trim(), rate) as TimeResult<Flicks, TimecodeErrorCode>;
}

/**
 * `time` as `HH:MM:SS:FF` at `rate`, like `se_domain::format_timecode`: the displayed frame
 * (rounded down), frames counted at the nominal rate (30 for 29.97), `-` before negative times.
 */
export function formatTimecode(time: Flicks, rate: FrameRate): string {
  const sign = time < 0 ? '-' : '';
  const frames = flicksToFrames(Math.abs(time) as Flicks, rate, 'floor');
  const nominal = nominalFps(rate);
  const frame = frames % nominal;
  const totalSeconds = Math.floor(frames / nominal);
  const pad = (value: number) => String(value).padStart(2, '0');
  const hours = Math.floor(totalSeconds / 3600);
  const minutes = Math.floor(totalSeconds / 60) % 60;
  return `${sign}${pad(hours)}:${pad(minutes)}:${pad(totalSeconds % 60)}:${pad(frame)}`;
}

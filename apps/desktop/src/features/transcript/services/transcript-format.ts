import type { Flicks } from '@se/protocol';

import { FLICKS_PER_SECOND } from '../../../shared/time';

/** A segment's start as `m:ss` (or `h:mm:ss` past an hour), for the paragraph labels. */
export function segmentClock(time: Flicks): string {
  const total = Math.max(0, Math.floor(time / FLICKS_PER_SECOND));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = String(total % 60).padStart(2, '0');
  const clock = `${String(minutes)}:${seconds}`;
  return hours > 0 ? `${String(hours)}:${clock.padStart(5, '0')}` : clock;
}

/** Languages offered for transcription (ISO 639-1), most used first; any other code is kept. */
export const TRANSCRIPTION_LANGUAGES = [
  'fr',
  'en',
  'es',
  'de',
  'it',
  'pt',
  'nl',
  'ar',
  'ru',
  'ja',
  'zh',
  'ko',
] as const;

/** The name of a language in French (`fr` → « français »), or the code when unknown. */
export function languageName(code: string): string {
  try {
    return new Intl.DisplayNames(['fr'], { type: 'language' }).of(code) ?? code;
  } catch {
    return code;
  }
}

/** The vocabulary as the core accepts it: one line, single spaces, at most 600 characters. */
export function cleanVocabulary(text: string): string {
  return text.replace(/\s+/g, ' ').trim().slice(0, 600);
}

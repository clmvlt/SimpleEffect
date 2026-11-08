import type { Flicks } from '@se/protocol';

import { flicksToSeconds } from '../../../shared/time';

/** `flicks` as French seconds with one decimal (`2,4`), for cut summaries and silence marks. */
export function cutSeconds(flicks: Flicks): string {
  return (Math.round(flicksToSeconds(flicks) * 10) / 10).toLocaleString('fr-FR', {
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  });
}

/** The excerpt of a struck passage: its first words, with an ellipsis past `limit` words. */
export function passageExcerpt(words: readonly { readonly text: string }[], limit = 12): string {
  const shown = words.slice(0, limit).map((word) => word.text);
  return words.length > limit ? `${shown.join(' ')} …` : shown.join(' ');
}

/**
 * Greedy line breaking of caption words by characters: the mirror of
 * `se_domain::captions::caption_lines`, run on `docs/test-vectors/caption-lines.json` by both
 * sides (change them together).
 */

/** Words `[start, end)` of one line. */
export type LineRange = readonly [start: number, end: number];

/** Characters of `text` as the core counts them (Unicode scalar values). */
export function lineChars(text: string): number {
  return Array.from(text).length;
}

/**
 * The lines of `words` when each holds at most `maxChars` characters, spaces between words
 * included: words are added while they fit; a word longer than `maxChars` gets its own line.
 */
export function captionLines(words: readonly string[], maxChars: number): LineRange[] {
  const lines: LineRange[] = [];
  let start = 0;
  let length = 0;
  words.forEach((word, index) => {
    const chars = lineChars(word);
    if (index > start && length + 1 + chars > maxChars) {
      lines.push([start, index]);
      start = index;
      length = chars;
    } else if (index === start) {
      length = chars;
    } else {
      length += 1 + chars;
    }
  });
  if (start < words.length) lines.push([start, words.length]);
  return lines;
}

/**
 * The characters (grapheme clusters, so « é » or an emoji count once) of `text`, in reading
 * order.
 */
export function graphemes(text: string): string[] {
  if (typeof Intl.Segmenter === 'function') {
    const segmenter = new Intl.Segmenter(undefined, { granularity: 'grapheme' });
    return Array.from(segmenter.segment(text), (segment) => segment.segment);
  }
  return Array.from(text);
}

/**
 * The part of each laid-out line shown when `reveal` (0 to 1) of the characters are: the first
 * `round(reveal × count)` characters of the block in reading order, line after line. Lines keep
 * their place (fully hidden lines become empty strings), so revealed characters never move.
 */
export function revealedLines(lines: readonly string[], reveal: number): string[] {
  if (reveal >= 1) return [...lines];
  const split = lines.map(graphemes);
  const total = split.reduce((count, line) => count + line.length, 0);
  let remaining = Math.round(Math.max(0, reveal) * total);
  return split.map((line) => {
    const shown = Math.min(line.length, remaining);
    remaining -= shown;
    return line.slice(0, shown).join('');
  });
}

/**
 * Where a line of `lineWidth` starts in a block of `blockWidth` for an alignment: lines are laid
 * out at their full width even when partly revealed.
 */
export function lineOffset(
  align: 'left' | 'center' | 'right',
  blockWidth: number,
  lineWidth: number,
): number {
  switch (align) {
    case 'left':
      return 0;
    case 'center':
      return (blockWidth - lineWidth) / 2;
    case 'right':
      return blockWidth - lineWidth;
  }
}

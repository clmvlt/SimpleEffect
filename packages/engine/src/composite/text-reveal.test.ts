import { describe, expect, it } from 'vitest';

import { graphemes, lineOffset, revealedLines } from './text-reveal';

describe('text reveal', () => {
  it('counts grapheme clusters, not code units', () => {
    expect(graphemes('été 👋🏽')).toEqual(['é', 't', 'é', ' ', '👋🏽']);
  });

  it('reveals characters in reading order, line after line, keeping hidden lines', () => {
    const lines = ['Bonjour', 'à tous'];
    expect(revealedLines(lines, 0)).toEqual(['', '']);
    expect(revealedLines(lines, 0.5)).toEqual(['Bonjour', '']);
    expect(revealedLines(lines, 10 / 13)).toEqual(['Bonjour', 'à t']);
    expect(revealedLines(lines, 1)).toEqual(lines);
  });

  it('aligns lines at their full width', () => {
    expect(lineOffset('left', 100, 40)).toBe(0);
    expect(lineOffset('center', 100, 40)).toBe(30);
    expect(lineOffset('right', 100, 40)).toBe(60);
  });
});

import type { CaptionItem, CaptionStyle, CaptionWord, Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { captionLayout, estimatedCaptionBox } from '../composite/caption-layout';
import { CAPTION_STYLE_PRESETS, captionLines, presetStyle } from '../caption-styles';
import vectors from '../../../../docs/test-vectors/caption-lines.json';
import { seconds } from '../testing/document';
import {
  POP_AMOUNT,
  activeWordIndex,
  captionLayerStyle,
  captionPage,
  captionText,
} from './caption-layer';

const ms = (value: number) => ((seconds(1) * value) / 1000) as Flicks;

function word(text: string, start: number, end: number, emphasis = false): CaptionWord {
  return { text, start: ms(start), end: ms(end), emphasis };
}

const WORDS = [
  word('Bonjour', 0, 400),
  word('à', 450, 600),
  word('tous', 650, 1000, true),
  word('et', 1100, 1200),
  word('bienvenue', 1250, 1800),
];

const CAPTION: CaptionItem = { words: WORDS, styleId: 'style' };

function style(overrides: Partial<CaptionStyle> = {}): CaptionStyle {
  return { ...presetStyle('karaoke', 'style'), ...overrides };
}

function layerAt(time: number, overrides: Partial<CaptionStyle> = {}) {
  const layer = captionLayerStyle(CAPTION, style(overrides), {
    localTime: ms(time),
    frame: { width: 1920, height: 1080 },
    designScale: 1,
  });
  if (layer === null) throw new Error('a caption with words draws something');
  return layer;
}

const WHITE = { r: 1, g: 1, b: 1, a: 1 };
const YELLOW = { r: 1, g: 214 / 255, b: 0, a: 1 };

describe('caption lines', () => {
  it('break like the core on the shared vectors', () => {
    for (const vector of vectors.cases) {
      expect(captionLines(vector.words, vector.maxChars), vector.name).toEqual(vector.lines);
    }
  });

  it('every preset is a valid style the default list holds', () => {
    expect(CAPTION_STYLE_PRESETS.map((preset) => preset.id)).toEqual([
      'tiktokBold',
      'minimal',
      'karaoke',
      'box',
    ]);
  });
});

describe('caption layers', () => {
  it('make the last started word active until the next one starts', () => {
    expect(activeWordIndex(WORDS, ms(-1))).toBe(-1);
    expect(activeWordIndex(WORDS, ms(0))).toBe(0);
    // In the pause after « Bonjour », it stays active.
    expect(activeWordIndex(WORDS, ms(420))).toBe(0);
    expect(activeWordIndex(WORDS, ms(1300))).toBe(4);
    expect(activeWordIndex(WORDS, ms(9000))).toBe(4);
  });

  it('show the page of lines holding the active word', () => {
    const lines = captionLines(['un', 'deux', 'trois', 'quatre'], 6);
    expect(lines).toEqual([
      [0, 1],
      [1, 2],
      [2, 3],
      [3, 4],
    ]);
    expect(captionPage(lines, 2, -1)).toEqual([
      [0, 1],
      [1, 2],
    ]);
    expect(captionPage(lines, 2, 2)).toEqual([
      [2, 3],
      [3, 4],
    ]);
  });

  it('highlight the spoken word, karaoke fills spoken ones, emphasis colours the rest', () => {
    const karaoke = layerAt(700).lines.flat();
    expect(karaoke.map((entry) => entry.color)).toEqual([YELLOW, YELLOW, YELLOW, WHITE, WHITE]);
    const highlight = layerAt(700, {
      highlightSpoken: false,
      emphasisColor: '#ff0000',
    }).lines.flat();
    expect(highlight[0]?.color).toEqual(WHITE);
    expect(highlight[2]?.color).toEqual(YELLOW);
    const later = layerAt(1150, { highlightSpoken: false, emphasisColor: '#ff0000' }).lines.flat();
    expect(later[2]?.color).toEqual({ r: 1, g: 0, b: 0, a: 1 });
    const still = layerAt(700, { animation: 'none' }).lines.flat();
    expect(still.every((entry) => entry.scale === 1 && entry.opacity === 1)).toBe(true);
  });

  it('pop each word as it starts and reveal words as they are spoken', () => {
    const pop = layerAt(450 + 90, { animation: 'popPerWord' }).lines.flat();
    expect(pop[1]?.scale).toBeCloseTo(1 + POP_AMOUNT, 5);
    expect(pop[0]?.scale).toBe(1);
    const reveal = layerAt(470, { animation: 'revealPerWord' }).lines.flat();
    expect(reveal.map((entry) => entry.opacity)).toEqual([1, 0.25, 0, 0, 0]);
  });

  it('apply the letter case and keep the text as typed', () => {
    const upper = layerAt(0, { textCase: 'uppercase' }).lines.flat();
    expect(upper[0]?.text).toBe('BONJOUR');
    expect(captionText(CAPTION)).toBe('Bonjour à tous et bienvenue');
  });

  it('give each line its own plate with boxPerLine', () => {
    const boxed = layerAt(0, { animation: 'boxPerLine', maxCharsPerLine: 14 });
    expect(boxed.lines.map((line) => line.map((entry) => entry.text))).toEqual([
      ['Bonjour', 'à', 'tous'],
      ['et', 'bienvenue'],
    ]);
    const layout = captionLayout(boxed, (text) => text.length * 10, 5, 40);
    expect(layout.plates).toHaveLength(2);
    expect(layout.words[0]?.centerY).toBe(20);
    expect(layout.words[3]?.centerY).toBe(60);
    // Line 1 (« Bonjour à tous », 120 px) is the widest: it starts at the block's left edge.
    expect(layout.words[0]?.centerX).toBe(35);
    expect(estimatedCaptionBox(boxed).height).toBeGreaterThan(0);
  });

  it('draw nothing without words', () => {
    expect(
      captionLayerStyle({ words: [], styleId: 'style' }, style(), {
        localTime: ms(0),
        frame: { width: 1920, height: 1080 },
        designScale: 1,
      }),
    ).toBeNull();
  });
});

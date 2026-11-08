import type { CaptionItem, CaptionStyle, CaptionWord, Flicks } from '@se/protocol';

import { BLACK, parseColor, type Rgba } from '../animation/color';
import { captionLines, type LineRange } from '../caption-styles/caption-lines';
import { FLICKS_PER_SECOND } from '../time/flicks';
import type { CaptionLayer, CaptionWordLayer, ResolvedTransform } from './frame-graph';
import type { GraphicContext } from './graphic-layers';

/** How long a word pops after it starts (seconds). */
export const POP_SECONDS = 0.18;
/** How much bigger a popping word gets at most (a share of its size). */
export const POP_AMOUNT = 0.18;
/** How long a word takes to appear with `revealPerWord` (seconds). */
export const REVEAL_SECONDS = 0.08;

/** A caption's text: its words joined by spaces (as typed, before any letter case). */
export function captionText(caption: Pick<CaptionItem, 'words'>): string {
  return caption.words.map((word) => word.text).join(' ');
}

/**
 * The word being spoken at `localTime` (item-relative): the last one whose start has passed,
 * which stays active until the next word starts; −1 before the first word.
 */
export function activeWordIndex(words: readonly CaptionWord[], localTime: Flicks): number {
  let active = -1;
  words.forEach((word, index) => {
    if (word.start <= localTime) active = index;
  });
  return active;
}

/**
 * The lines shown: the page of `maxLines` lines holding the active word (the first page before
 * any word is spoken).
 */
export function captionPage(
  lines: readonly LineRange[],
  maxLines: number,
  active: number,
): LineRange[] {
  const perPage = Math.max(1, maxLines);
  const activeLine = Math.max(
    0,
    lines.findIndex(([start, end]) => active >= start && active < end),
  );
  const first = Math.floor(activeLine / perPage) * perPage;
  return lines.slice(first, first + perPage);
}

function color(text: string): Rgba {
  return parseColor(text) ?? BLACK;
}

/** The colour of word `index` when word `active` is spoken. */
function wordColor(style: CaptionStyle, word: CaptionWord, index: number, active: number): Rgba {
  if (style.animation !== 'none') {
    if (index === active) return color(style.activeColor);
    if (style.highlightSpoken && index < active) return color(style.activeColor);
  }
  return color(word.emphasis ? style.emphasisColor : style.textColor);
}

/** A share in `[0, 1]` of the way through `seconds` after `start`. */
function progressSince(start: Flicks, localTime: Flicks, seconds: number): number {
  const elapsed = (localTime - start) / FLICKS_PER_SECOND;
  return Math.min(1, Math.max(0, elapsed / seconds));
}

function wordLayer(
  style: CaptionStyle,
  word: CaptionWord,
  index: number,
  active: number,
  localTime: Flicks,
): CaptionWordLayer {
  const started = word.start <= localTime;
  let scale = 1;
  if (style.animation === 'popPerWord' && started) {
    const progress = progressSince(word.start, localTime, POP_SECONDS);
    scale = 1 + POP_AMOUNT * Math.sin(Math.PI * progress);
  }
  const opacity =
    style.animation === 'revealPerWord'
      ? started
        ? progressSince(word.start, localTime, REVEAL_SECONDS)
        : 0
      : 1;
  return {
    text: style.textCase === 'uppercase' ? word.text.toUpperCase() : word.text,
    color: wordColor(style, word, index, active),
    scale,
    opacity,
  };
}

/**
 * Everything of a caption layer but its item id and transform, at the context's time; `null`
 * for a caption without words.
 */
export function captionLayerStyle(
  caption: CaptionItem,
  style: CaptionStyle,
  context: GraphicContext,
): Omit<CaptionLayer, 'itemId' | 'transform' | 'dissolve'> | null {
  const { words } = caption;
  if (words.length === 0) return null;
  const time = context.localTime;
  const scale = context.designScale;
  const active = activeWordIndex(words, time);
  const texts = words.map((word) =>
    style.textCase === 'uppercase' ? word.text.toUpperCase() : word.text,
  );
  const page = captionPage(captionLines(texts, style.maxCharsPerLine), style.maxLines, active);
  const lines = page.map(([start, end]) =>
    words
      .slice(start, end)
      .map((word, offset) => wordLayer(style, word, start + offset, active, time)),
  );
  const { outline, shadow, background } = style;
  return {
    kind: 'caption',
    lines,
    font: { family: style.fontFamily, weight: style.fontWeight, italic: style.italic },
    fontSize: Math.max(0, style.fontSize) * scale,
    lineHeight: style.lineHeight,
    stroke:
      outline.enabled && outline.width > 0
        ? { color: color(outline.color), width: outline.width * scale }
        : null,
    shadow: shadow.enabled
      ? {
          color: color(shadow.color),
          blur: Math.max(0, shadow.blur) * scale,
          offset: { x: shadow.offset.x * scale, y: shadow.offset.y * scale },
        }
      : null,
    background:
      background.enabled || style.animation === 'boxPerLine'
        ? {
            color: color(background.color),
            padding: {
              x: Math.max(0, background.padding.x) * scale,
              y: Math.max(0, background.padding.y) * scale,
            },
            cornerRadius: Math.max(0, background.cornerRadius) * scale,
          }
        : null,
    boxPerLine: style.animation === 'boxPerLine',
  };
}

/** Where a caption sits: its block's bottom centre at the style's position, unscaled. */
export function captionTransform(style: CaptionStyle, opacity: number): ResolvedTransform {
  return {
    position: style.position,
    scale: { x: 1, y: 1 },
    rotation: 0,
    anchor: { x: 0.5, y: 1 },
    opacity,
    blur: 0,
  };
}

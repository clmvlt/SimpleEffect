import type { Size, TextLayer } from '@se/engine/evaluate';

/** The box a text layer takes in sequence pixels (plate and outline included). */
export type TextMeasurer = (layer: TextLayer) => Size;

/** Measures a run of text in pixels at the layer's font (letter spacing excluded). */
type RunWidth = (text: string) => number;

/** Average advance of a glyph as a share of the font size, when nothing can measure. */
const AVERAGE_ADVANCE = 0.55;

const graphemes = new Intl.Segmenter(undefined, { granularity: 'grapheme' });

/** Characters as a reader counts them (grapheme clusters: an emoji with modifiers is one). */
function characterCount(text: string): number {
  return Array.from(graphemes.segment(text)).length;
}

/** The lines of `content` once wrapped greedily at word boundaries within `maxWidth`. */
function wrappedLines(content: string, maxWidth: number | null, width: RunWidth): string[] {
  const lines: string[] = [];
  for (const paragraph of content.split('\n')) {
    if (maxWidth === null) {
      lines.push(paragraph);
      continue;
    }
    let line = '';
    for (const word of paragraph.split(' ')) {
      const candidate = line === '' ? word : `${line} ${word}`;
      if (line !== '' && width(candidate) > maxWidth) {
        lines.push(line);
        line = word;
      } else {
        line = candidate;
      }
    }
    lines.push(line);
  }
  return lines;
}

/**
 * The box of a text layer laid out with `width` (lines wrapped like the compositor's, letter
 * spacing between characters, line height times the font size, outline and plate around): close
 * enough to tell whether a text leaves the frame, without drawing it.
 */
export function measureTextLayer(layer: TextLayer, width: RunWidth): Size {
  const spaced = (text: string) =>
    width(text) + layer.letterSpacing * Math.max(0, characterCount(text) - 1);
  const lines = wrappedLines(layer.content, layer.maxWidth, spaced);
  const textWidth = Math.max(0, ...lines.map(spaced));
  const textHeight = lines.length * layer.lineHeight * layer.fontSize;
  const outline = 2 * (layer.stroke?.width ?? 0);
  const padding = layer.background?.padding ?? { x: 0, y: 0 };
  return {
    width: textWidth + outline + 2 * padding.x,
    height: textHeight + outline + 2 * padding.y,
  };
}

/** An estimate from the average glyph advance (no canvas available, e.g. in tests). */
export function estimateTextLayer(layer: TextLayer): Size {
  return measureTextLayer(layer, (text) => characterCount(text) * AVERAGE_ADVANCE * layer.fontSize);
}

/**
 * Measures text layers with a 2D canvas and the fonts the document has loaded (the catalogue's
 * faces registered for the viewer); falls back to {@link estimateTextLayer} without a canvas.
 */
export function createCanvasTextMeasurer(): TextMeasurer {
  const context =
    typeof document === 'undefined' ? null : document.createElement('canvas').getContext('2d');
  if (context === null) return estimateTextLayer;
  return (layer) => {
    const { family, weight, italic } = layer.font;
    context.font = `${italic ? 'italic ' : ''}${String(weight)} ${String(layer.fontSize)}px "${family}", sans-serif`;
    return measureTextLayer(layer, (text) => context.measureText(text).width);
  };
}

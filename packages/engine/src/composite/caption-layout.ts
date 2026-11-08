import type { CaptionLayer, Size } from '../evaluate/frame-graph';

/** Share of a line's height left between the plates of two lines (`boxPerLine`). */
export const LINE_PLATE_GAP = 0.08;

/** Where one word of a caption is drawn, relative to the block (padding excluded). */
export interface CaptionWordPlace {
  readonly centerX: number;
  readonly centerY: number;
  readonly width: number;
}

/** A plate rectangle in the caption's box. */
export interface CaptionPlate {
  readonly x: number;
  readonly y: number;
  readonly width: number;
  readonly height: number;
}

/** A caption's geometry, in sequence pixels. */
export interface CaptionLayout {
  /** Every word of every line, in reading order. */
  readonly words: readonly CaptionWordPlace[];
  /** The box (block plus padding when a plate is drawn): what the layer occupies. */
  readonly box: Size;
  /** Offset of the block in the box. */
  readonly padding: { readonly x: number; readonly y: number };
  /** The plates to draw: one for the block, one per line with `boxPerLine`, none without plate. */
  readonly plates: readonly CaptionPlate[];
}

/**
 * The geometry of `layer`'s lines, given how wide a word is (`widthOf`), a space and a line:
 * lines are centred under each other; each word sits at its natural place. Pure, so the
 * compositor and tests share it.
 */
export function captionLayout(
  layer: Pick<CaptionLayer, 'lines' | 'background' | 'boxPerLine'>,
  widthOf: (text: string) => number,
  space: number,
  lineHeight: number,
): CaptionLayout {
  const lines = layer.lines.map((line) => {
    const widths = line.map((word) => widthOf(word.text));
    const total =
      widths.reduce((sum, width) => sum + width, 0) + space * Math.max(0, line.length - 1);
    return { widths, total };
  });
  const blockWidth = Math.max(0, ...lines.map((line) => line.total));
  const padding = layer.background?.padding ?? { x: 0, y: 0 };
  const words: CaptionWordPlace[] = [];
  const plates: CaptionPlate[] = [];
  lines.forEach((line, index) => {
    let x = (blockWidth - line.total) / 2;
    const centerY = index * lineHeight + lineHeight / 2;
    line.widths.forEach((width) => {
      words.push({ centerX: x + width / 2, centerY, width });
      x += width + space;
    });
    if (layer.background !== null && layer.boxPerLine) {
      // Line plates span their line (a small gap apart, never overlapping: translucent plates
      // would darken where they meet), widened by the horizontal padding.
      const gap = lineHeight * LINE_PLATE_GAP;
      plates.push({
        x: (blockWidth - line.total) / 2,
        y: padding.y + index * lineHeight + gap / 2,
        width: line.total + padding.x * 2,
        height: lineHeight - gap,
      });
    }
  });
  const box = {
    width: blockWidth + padding.x * 2,
    height: lineHeight * Math.max(1, lines.length) + padding.y * 2,
  };
  if (layer.background !== null && !layer.boxPerLine) {
    plates.push({ x: 0, y: 0, ...box });
  }
  return { words, box, padding, plates };
}

/**
 * A caption's box estimated without fonts (0.6 em per character, a space as 0.3 em): for tests
 * and for checks made outside a compositor (texts are measured only while drawing).
 */
export function estimatedCaptionBox(
  layer: Pick<CaptionLayer, 'lines' | 'background' | 'boxPerLine' | 'fontSize' | 'lineHeight'>,
): Size {
  const em = layer.fontSize;
  return captionLayout(
    layer,
    (text) => Array.from(text).length * em * 0.6,
    em * 0.3,
    em * layer.lineHeight,
  ).box;
}

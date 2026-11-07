import { CanvasTextMetrics, Container, Graphics, Text, TextStyle } from 'pixi.js';

import type { ResolvedBackground, Size, TextLayer } from '../../evaluate/frame-graph';
import { lineOffset, revealedLines } from '../text-reveal';
import { LayerBlur, pixiColor, place, type LayerNode } from './layer-node';

/**
 * Fonts tried after the layer's family: the bundled Inter (font catalogue), the UI's copy of it,
 * then the platform's sans-serif. A missing family is reported by the frame renderer.
 */
export const FALLBACK_FONTS = ['Inter', 'Inter Variable', 'sans-serif'];

/** The lines of a text block and their widths, measured without outline or shadow. */
interface TextLayout {
  readonly lines: readonly string[];
  readonly widths: readonly number[];
  readonly width: number;
  readonly lineHeight: number;
}

/** The font part of a Pixi text style (shared by measuring and drawing). */
function fontOptions(layer: TextLayer) {
  return {
    fontFamily: [layer.font.family, ...FALLBACK_FONTS],
    fontSize: Math.max(0.01, layer.fontSize),
    fontWeight: String(Math.round(layer.font.weight)) as TextStyle['fontWeight'],
    fontStyle: layer.font.italic ? ('italic' as const) : ('normal' as const),
    letterSpacing: layer.letterSpacing,
    lineHeight: Math.max(0.01, layer.fontSize * layer.lineHeight),
  };
}

/** The drawing style of a line: font, fill, outline (all of it outside) and shadow. */
function drawingOptions(layer: TextLayer) {
  const shadow = layer.shadow;
  const stroke = layer.stroke;
  return {
    ...fontOptions(layer),
    fill: pixiColor(layer.color),
    // Pixi strokes on the glyph outline (half inside, covered by the fill): twice the width
    // shows the requested width outside. No outline is a stroke of no width.
    stroke:
      stroke === null
        ? { color: 0, alpha: 0, width: 0 }
        : { ...pixiColor(stroke.color), width: stroke.width * 2, join: 'round' as const },
    dropShadow:
      shadow === null
        ? false
        : {
            ...pixiColor(shadow.color),
            blur: shadow.blur,
            distance: Math.hypot(shadow.offset.x, shadow.offset.y),
            angle: Math.atan2(shadow.offset.y, shadow.offset.x),
          },
    padding:
      shadow === null
        ? 0
        : Math.ceil(shadow.blur * 2 + Math.hypot(shadow.offset.x, shadow.offset.y)),
  };
}

/**
 * Styled text drawn line by line: the layout (wrapping, widths) is measured on the whole text
 * without outline or shadow, so revealing characters, outlining or shadowing never moves the
 * text; each line is a Pixi `Text` at its aligned place, over an optional plate.
 */
export class TextNode implements LayerNode {
  readonly kind = 'text';
  readonly root = new Container();
  private readonly plate = new Graphics();
  private readonly block = new Container();
  private readonly blur = new LayerBlur();
  private readonly style = new TextStyle();
  private texts: Text[] = [];
  private styleKey = '';
  private layoutKey = '';
  private layout: TextLayout = { lines: [], widths: [], width: 0, lineHeight: 0 };
  private plateKey = '';

  constructor() {
    this.root.addChild(this.plate, this.block);
  }

  /**
   * Draws `layer`; `scale` is output pixels per sequence pixel (glyphs are rasterised at it so
   * they stay sharp). Returns the layer's box: the text block, plus the plate when drawn.
   */
  update(layer: TextLayer, frame: Size, scale: number): Size {
    this.applyStyle(layer);
    const layout = this.measure(layer);
    const shown = revealedLines(layout.lines, layer.reveal);
    this.syncTexts(shown.length, Math.max(1, scale));
    const halfStroke = layer.stroke?.width ?? 0;
    shown.forEach((line, index) => {
      const text = this.texts[index];
      if (text === undefined) return;
      text.text = line;
      const offset = lineOffset(layer.align, layout.width, layout.widths[index] ?? 0);
      text.position.set(offset - halfStroke, index * layout.lineHeight - halfStroke);
    });
    const padding = layer.background?.padding ?? { x: 0, y: 0 };
    const box = {
      width: layout.width + padding.x * 2,
      height: layout.lineHeight * Math.max(1, layout.lines.length) + padding.y * 2,
    };
    this.block.position.set(padding.x, padding.y);
    this.drawPlate(layer.background, box);
    place(this.root, layer, box, frame);
    this.blur.apply(this.root, layer.transform.blur * scale);
    return box;
  }

  private applyStyle(layer: TextLayer): void {
    const options = drawingOptions(layer);
    const key = JSON.stringify(options);
    if (key === this.styleKey) return;
    this.styleKey = key;
    Object.assign(this.style, options);
  }

  private measure(layer: TextLayer): TextLayout {
    const options = {
      ...fontOptions(layer),
      wordWrap: layer.maxWidth !== null,
      wordWrapWidth: Math.max(1, layer.maxWidth ?? 0),
      breakWords: true,
    };
    const key = JSON.stringify([layer.content, options]);
    if (key === this.layoutKey) return this.layout;
    const metrics = CanvasTextMetrics.measureText(layer.content, new TextStyle(options));
    this.layoutKey = key;
    this.layout = {
      lines: metrics.lines,
      widths: metrics.lineWidths,
      width: metrics.lines.length === 0 ? 0 : metrics.maxLineWidth,
      lineHeight: metrics.lineHeight,
    };
    return this.layout;
  }

  /** Keeps one `Text` per line, all sharing the node's style. */
  private syncTexts(count: number, resolution: number): void {
    while (this.texts.length < count) {
      const text = new Text({ text: '', style: this.style });
      this.texts.push(text);
      this.block.addChild(text);
    }
    for (const extra of this.texts.splice(count)) extra.destroy();
    for (const text of this.texts) {
      if (text.resolution !== resolution) text.resolution = resolution;
    }
  }

  private drawPlate(background: ResolvedBackground | null, box: Size): void {
    const key = JSON.stringify([background, box]);
    if (key === this.plateKey) return;
    this.plateKey = key;
    this.plate.clear();
    if (background === null) return;
    const radius = Math.min(background.cornerRadius, box.width / 2, box.height / 2);
    this.plate.roundRect(0, 0, box.width, box.height, radius).fill(pixiColor(background.color));
  }

  destroy(): void {
    this.blur.destroy();
    this.root.destroy({ children: true });
    this.style.destroy();
  }
}

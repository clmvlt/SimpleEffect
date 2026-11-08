import { CanvasTextMetrics, Container, Graphics, Text, TextStyle } from 'pixi.js';

import type { Rgba } from '../../animation/color';
import type { CaptionLayer, Size } from '../../evaluate/frame-graph';
import { captionLayout, type CaptionLayout } from '../caption-layout';
import { pixiColor, place, type LayerNode } from './layer-node';
import { FALLBACK_FONTS } from './text-node';

/** The font part of a caption's Pixi text style (shared by measuring and drawing). */
function fontOptions(layer: CaptionLayer) {
  return {
    fontFamily: [layer.font.family, ...FALLBACK_FONTS],
    fontSize: Math.max(0.01, layer.fontSize),
    fontWeight: String(Math.round(layer.font.weight)) as TextStyle['fontWeight'],
    fontStyle: layer.font.italic ? ('italic' as const) : ('normal' as const),
    lineHeight: Math.max(0.01, layer.fontSize * layer.lineHeight),
  };
}

/** The drawing style of words of one colour: font, fill, outline (all outside) and shadow. */
function drawingOptions(layer: CaptionLayer, color: Rgba) {
  const { stroke, shadow } = layer;
  return {
    ...fontOptions(layer),
    fill: pixiColor(color),
    // Pixi strokes on the glyph outline (half inside): twice the width shows it all outside.
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
        ? Math.ceil(stroke?.width ?? 0)
        : Math.ceil(shadow.blur * 2 + Math.hypot(shadow.offset.x, shadow.offset.y)),
  };
}

/**
 * A caption drawn word by word: the layout (word widths, spaces, centred lines) is measured on the
 * plain words, so colours, pops, reveals and outlines never move a word; each word is a Pixi
 * `Text` centred on its place (pops scale it there), over a plate for the block or one per line.
 */
export class CaptionNode implements LayerNode {
  readonly kind = 'caption';
  readonly root = new Container();
  private readonly plates = new Graphics();
  private readonly block = new Container();
  private texts: Text[] = [];
  private readonly styles = new Map<string, TextStyle>();
  private layoutKey = '';
  private layout: CaptionLayout | null = null;
  private platesKey = '';

  constructor() {
    this.root.addChild(this.plates, this.block);
  }

  /** Draws `layer` (`scale`: output pixels per sequence pixel); returns its box. */
  update(layer: CaptionLayer, frame: Size, scale: number): Size {
    const layout = this.measure(layer);
    const words = layer.lines.flat();
    this.syncTexts(words.length, Math.max(1, scale));
    words.forEach((word, index) => {
      const text = this.texts[index];
      const place = layout.words[index];
      if (text === undefined || place === undefined) return;
      text.text = word.text;
      text.style = this.styleFor(layer, word.color);
      text.position.set(place.centerX, place.centerY);
      text.scale.set(word.scale);
      text.alpha = word.opacity;
    });
    this.block.position.set(layout.padding.x, layout.padding.y);
    this.drawPlates(layer, layout);
    place(this.root, layer, layout.box, frame);
    return layout.box;
  }

  private measure(layer: CaptionLayer): CaptionLayout {
    const options = fontOptions(layer);
    const key = JSON.stringify([
      layer.lines.map((line) => line.map((word) => word.text)),
      options,
      layer.background?.padding ?? null,
    ]);
    if (key === this.layoutKey && this.layout !== null) return this.layout;
    const style = new TextStyle(options);
    const width = (text: string) => CanvasTextMetrics.measureText(text, style).width;
    const space = Math.max(0, width('x x') - width('xx'));
    this.layout = captionLayout(layer, width, space, options.lineHeight);
    this.layoutKey = key;
    style.destroy();
    return this.layout;
  }

  /** One shared style per colour (Pixi re-rasterises a text whose style changes). */
  private styleFor(layer: CaptionLayer, color: Rgba): TextStyle {
    const options = drawingOptions(layer, color);
    const key = JSON.stringify(options);
    let style = this.styles.get(key);
    if (style === undefined) {
      style = new TextStyle(options);
      this.styles.set(key, style);
    }
    return style;
  }

  private syncTexts(count: number, resolution: number): void {
    while (this.texts.length < count) {
      const text = new Text({ text: '', anchor: 0.5 });
      this.texts.push(text);
      this.block.addChild(text);
    }
    for (const extra of this.texts.splice(count)) extra.destroy();
    for (const text of this.texts) {
      if (text.resolution !== resolution) text.resolution = resolution;
    }
  }

  private drawPlates(layer: CaptionLayer, layout: CaptionLayout): void {
    const key = JSON.stringify([layer.background, layer.boxPerLine, layout.plates]);
    if (key === this.platesKey) return;
    this.platesKey = key;
    this.plates.clear();
    const background = layer.background;
    if (background === null) return;
    const fill = pixiColor(background.color);
    for (const plate of layout.plates) {
      const radius = Math.min(background.cornerRadius, plate.width / 2, plate.height / 2);
      this.plates.roundRect(plate.x, plate.y, plate.width, plate.height, radius).fill(fill);
    }
  }

  destroy(): void {
    this.root.destroy({ children: true });
    for (const style of this.styles.values()) style.destroy();
    this.styles.clear();
  }
}

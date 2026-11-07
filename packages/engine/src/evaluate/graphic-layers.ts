import type { Flicks, ShapeItem, Stroke, TextItem, Vec2 } from '@se/protocol';

import { BLACK, parseColor, type Rgba } from '../animation/color';
import {
  interpolateColor,
  interpolateNumber,
  interpolateVec2,
  valueAt,
} from '../animation/interpolate';
import type {
  ResolvedBackground,
  ResolvedShadow,
  ResolvedStroke,
  ShapeLayer,
  Size,
  TextLayer,
} from './frame-graph';
import { shapeBox } from './geometry';

/** What resolving a graphic needs: the item-relative time, the frame and the design scale. */
export interface GraphicContext {
  readonly localTime: Flicks;
  readonly frame: Size;
  readonly designScale: number;
}

const ORIGIN: Vec2 = { x: 0, y: 0 };

function colorAt(property: TextItem['style']['color'], time: Flicks): Rgba {
  return parseColor(valueAt(property, time, interpolateColor, '#000000')) ?? BLACK;
}

function numberAt(property: TextItem['reveal'], time: Flicks, fallback: number): number {
  return valueAt(property, time, interpolateNumber, fallback);
}

/** An enabled outline in sequence pixels, or `null` when off or of no width. */
export function resolveStroke(stroke: Stroke, context: GraphicContext): ResolvedStroke | null {
  if (!stroke.enabled) return null;
  const width = Math.max(0, numberAt(stroke.width, context.localTime, 0)) * context.designScale;
  if (width <= 0) return null;
  return { color: colorAt(stroke.color, context.localTime), width };
}

function resolveShadow(style: TextItem['style'], context: GraphicContext): ResolvedShadow | null {
  const { shadow } = style;
  if (!shadow.enabled) return null;
  const offset = valueAt(shadow.offset, context.localTime, interpolateVec2, ORIGIN);
  return {
    color: colorAt(shadow.color, context.localTime),
    blur: Math.max(0, numberAt(shadow.blur, context.localTime, 0)) * context.designScale,
    offset: { x: offset.x * context.designScale, y: offset.y * context.designScale },
  };
}

function resolveBackground(
  style: TextItem['style'],
  context: GraphicContext,
): ResolvedBackground | null {
  const { background } = style;
  if (!background.enabled) return null;
  const scale = context.designScale;
  return {
    color: colorAt(background.color, context.localTime),
    padding: {
      x: Math.max(0, background.padding.x) * scale,
      y: Math.max(0, background.padding.y) * scale,
    },
    cornerRadius: Math.max(0, background.cornerRadius) * scale,
  };
}

/** Everything of a text layer but its item id and transform, at the context's time. */
export function textLayerStyle(
  text: TextItem,
  context: GraphicContext,
): Omit<TextLayer, 'itemId' | 'transform' | 'dissolve'> {
  const { style } = text;
  const time = context.localTime;
  const scale = context.designScale;
  return {
    kind: 'text',
    content: style.textCase === 'uppercase' ? text.content.toUpperCase() : text.content,
    font: { family: style.fontFamily, weight: style.fontWeight, italic: style.italic },
    fontSize: Math.max(0, numberAt(style.fontSize, time, 0)) * scale,
    color: colorAt(style.color, time),
    letterSpacing: numberAt(style.letterSpacing, time, 0) * scale,
    lineHeight: style.lineHeight,
    align: style.align,
    maxWidth: style.maxWidth === null ? null : style.maxWidth * scale,
    stroke: resolveStroke(style.stroke, context),
    shadow: resolveShadow(style, context),
    background: resolveBackground(style, context),
    reveal: Math.min(1, Math.max(0, numberAt(text.reveal, time, 1))),
  };
}

/** Everything of a shape layer but its item id and transform, at the context's time. */
export function shapeLayerStyle(
  shape: ShapeItem,
  context: GraphicContext,
): Omit<ShapeLayer, 'itemId' | 'transform' | 'dissolve'> {
  const { box, cornerRadius } = shapeBox(shape.geometry, context.designScale, {
    frame: context.frame,
    coverFrame: shape.coverFrame,
  });
  return {
    kind: 'shape',
    geometry: shape.geometry.kind,
    box,
    cornerRadius,
    fill: colorAt(shape.fill, context.localTime),
    stroke: resolveStroke(shape.stroke, context),
  };
}

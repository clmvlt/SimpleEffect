import type { Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import {
  assetTable,
  imageAsset,
  image,
  item,
  line,
  rectangle,
  seconds,
  sequence,
  still,
  stroke,
  text,
  textStyle,
  videoTrack,
} from '../testing/document';
import { evaluate } from './evaluate';
import type { Layer, ShapeLayer, TextLayer } from './frame-graph';

const at = (kind: Parameters<typeof item>[2]) => item(0 as Flicks, seconds(2), kind);

function layerAt(width: number, height: number, kind: Parameters<typeof item>[2], time = 0) {
  const graph = evaluate(sequence(width, height, [videoTrack([at(kind)])]), {}, seconds(time));
  const layer = graph.layers[0];
  if (layer === undefined) throw new Error('nothing drawn');
  return layer;
}

function textAt(...args: Parameters<typeof layerAt>): TextLayer {
  const layer: Layer = layerAt(...args);
  if (layer.kind !== 'text') throw new Error('expected a text layer');
  return layer;
}

function shapeAt(...args: Parameters<typeof layerAt>): ShapeLayer {
  const layer: Layer = layerAt(...args);
  if (layer.kind !== 'shape') throw new Error('expected a shape layer');
  return layer;
}

describe('text layers', () => {
  it('resolve the whole style in sequence pixels', () => {
    const style = textStyle({
      fontFamily: 'Montserrat',
      fontWeight: 800,
      italic: true,
      letterSpacing: still(4),
      maxWidth: 900,
      align: 'left',
      textCase: 'uppercase',
      stroke: stroke('#ff0000', 3),
      shadow: {
        enabled: true,
        color: still('#00000080'),
        blur: still(10),
        offset: still({ x: 2, y: 6 }),
      },
      background: {
        enabled: true,
        color: still('#112233'),
        padding: { x: 20, y: 10 },
        cornerRadius: 6,
      },
    });
    const layer = textAt(3840, 2160, text('Salut', { style }));
    expect(layer).toMatchObject({
      content: 'SALUT',
      font: { family: 'Montserrat', weight: 800, italic: true },
      fontSize: 192,
      letterSpacing: 8,
      maxWidth: 1800,
      align: 'left',
      stroke: { width: 6, color: { r: 1, g: 0, b: 0, a: 1 } },
      shadow: { blur: 20, offset: { x: 4, y: 12 } },
      background: { padding: { x: 40, y: 20 }, cornerRadius: 12 },
      reveal: 1,
    });
  });

  it('leave disabled parts out and animate the reveal', () => {
    const reveal = {
      kind: 'keyframed' as const,
      keyframes: [
        { time: 0 as Flicks, value: 0, easing: { kind: 'linear' as const } },
        { time: seconds(1), value: 1, easing: { kind: 'linear' as const } },
      ],
    };
    const layer = textAt(1920, 1080, text('Bonjour', { reveal }), 0.25);
    expect(layer.stroke).toBeNull();
    expect(layer.shadow).toBeNull();
    expect(layer.background).toBeNull();
    expect(layer.reveal).toBeCloseTo(0.25);
  });
});

describe('shape layers', () => {
  it('draw lines as capsules and outlines in sequence pixels', () => {
    const layer = shapeAt(1280, 720, line('#ffffff', 600, 12, { stroke: stroke('#000000', 3) }));
    expect(layer).toMatchObject({
      geometry: 'line',
      box: { width: 400, height: 8 },
      cornerRadius: 4,
      stroke: { width: 2 },
    });
  });

  it('cover the frame when asked, whatever its format', () => {
    const plate = rectangle('#000000', 10, 10, { coverFrame: true });
    expect(shapeAt(1080, 1920, plate).box).toEqual({ width: 1080, height: 1920 });
  });

  it('animate their fill colour', () => {
    const fill = {
      kind: 'keyframed' as const,
      keyframes: [
        { time: 0 as Flicks, value: '#000000', easing: { kind: 'linear' as const } },
        { time: seconds(2), value: '#ffffff', easing: { kind: 'linear' as const } },
      ],
    };
    const start = shapeAt(1920, 1080, rectangle(fill, 10, 10), 0);
    const middle = shapeAt(1920, 1080, rectangle(fill, 10, 10), 1);
    expect(start.fill.r).toBe(0);
    expect(middle.fill.r).toBeGreaterThan(0.3);
    expect(middle.fill.r).toBeLessThan(0.9);
  });
});

describe('images and blur', () => {
  it('give natural images one design pixel per image pixel', () => {
    const logo = imageAsset(400, 200);
    const graph = evaluate(
      sequence(3840, 2160, [videoTrack([at(image(logo.id, {}, 'natural'))])]),
      assetTable(logo),
      0 as Flicks,
    );
    expect(graph.layers[0]).toMatchObject({ box: { width: 800, height: 400 } });
  });

  it('scale blur radii by the design scale', () => {
    const layer = shapeAt(3840, 2160, rectangle('#ffffff', 10, 10, { transform: { blur: 12 } }));
    expect(layer.transform.blur).toBe(24);
  });
});

import type { Flicks, Reframe } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import {
  NTSC_30,
  assetTable,
  clip,
  ellipse,
  frameTime,
  image,
  imageAsset,
  item,
  rectangle,
  seconds,
  sequence,
  still,
  text,
  videoAsset,
  videoTrack,
} from '../testing/document';
import { evaluate } from './evaluate';
import type { Layer, VideoLayer } from './frame-graph';

const video = videoAsset({ width: 1920, height: 1080 });
const assets = assetTable(video);

function onlyLayer(layers: readonly Layer[]): Layer {
  expect(layers).toHaveLength(1);
  const [layer] = layers;
  if (layer === undefined) throw new Error('no layer');
  return layer;
}

function videoLayer(layers: readonly Layer[]): VideoLayer {
  const layer = onlyLayer(layers);
  if (layer.kind !== 'video') throw new Error(`expected a video layer, got ${layer.kind}`);
  return layer;
}

describe('evaluate: visibility', () => {
  const red = item(seconds(1), seconds(2), rectangle('#ff0000', 100, 100));
  const scene = sequence(1920, 1080, [videoTrack([red])]);

  it('shows an item over its half-open range only', () => {
    expect(evaluate(scene, assets, seconds(0.99)).layers).toHaveLength(0);
    expect(evaluate(scene, assets, seconds(1)).layers).toHaveLength(1);
    expect(evaluate(scene, assets, (seconds(3) - 1) as Flicks).layers).toHaveLength(1);
    expect(evaluate(scene, assets, seconds(3)).layers).toHaveLength(0);
  });

  it('stacks video tracks bottom first and skips hidden and audio tracks', () => {
    const bottom = item(0 as Flicks, seconds(5), rectangle('#ff0000', 10, 10));
    const hidden = item(0 as Flicks, seconds(5), rectangle('#00ff00', 10, 10));
    const top = item(0 as Flicks, seconds(5), ellipse('#0000ff', 10, 10));
    const sound = item(0 as Flicks, seconds(5), clip(video.id));
    const layered = sequence(1920, 1080, [
      videoTrack([bottom]),
      videoTrack([hidden], true),
      { kind: 'audio', hidden: false, items: [sound] },
      videoTrack([top]),
    ]);
    const graph = evaluate(layered, assets, seconds(1));
    expect(graph.layers.map((layer) => layer.itemId)).toEqual([bottom.id, top.id]);
  });

  it('finds the item at the time among many by binary search', () => {
    const items = Array.from({ length: 200 }, (_, index) =>
      item(seconds(index), seconds(1), rectangle('#ffffff', 10, 10)),
    );
    const graph = evaluate(sequence(1920, 1080, [videoTrack(items)]), assets, seconds(123.5));
    expect(onlyLayer(graph.layers).itemId).toBe(items[123]?.id);
  });

  it('leaves out invisible items, templates and clips of unknown assets', () => {
    const transparent = item(
      0 as Flicks,
      seconds(1),
      rectangle('#ffffff', 10, 10, {
        transform: { opacity: 0 },
      }),
    );
    const flat = item(
      0 as Flicks,
      seconds(1),
      rectangle('#ffffff', 10, 10, {
        transform: { scale: { x: 0, y: 1 } },
      }),
    );
    const template = item(0 as Flicks, seconds(1), {
      kind: 'motionTemplate',
      templateId: 'template',
      params: {},
    });
    const orphan = item(0 as Flicks, seconds(1), clip('unknown-asset'));
    const tracks = [transparent, flat, template, orphan].map((entry) => videoTrack([entry]));
    expect(evaluate(sequence(1920, 1080, tracks), assets, 0 as Flicks).layers).toEqual([]);
  });
});

describe('evaluate: clip timing', () => {
  it('maps the sequence time to the source time', () => {
    const range = item(seconds(10), seconds(4), clip(video.id, { sourceStart: seconds(2) }));
    const layer = videoLayer(
      evaluate(sequence(1920, 1080, [videoTrack([range])]), assets, seconds(11.5)).layers,
    );
    expect(layer.sourceTime).toBe(seconds(3.5));
  });

  it('applies the speed to the time since the clip start', () => {
    const fast = item(
      seconds(1),
      seconds(2),
      clip(video.id, { sourceStart: seconds(1), speed: 2 }),
    );
    const slow = item(seconds(1), seconds(2), clip(video.id, { speed: 0.5 }));
    const time = seconds(1.5);
    const at = (entry: typeof fast) =>
      videoLayer(evaluate(sequence(1920, 1080, [videoTrack([entry])]), assets, time).layers);
    expect(at(fast).sourceTime).toBe(seconds(2));
    expect(at(slow).sourceTime).toBe(seconds(0.25));
  });

  it('keeps NTSC frame times exact', () => {
    const range = item(0 as Flicks, seconds(10), clip(video.id));
    const graph = evaluate(
      sequence(1920, 1080, [videoTrack([range])]),
      assets,
      frameTime(299, NTSC_30),
    );
    expect(videoLayer(graph.layers).sourceTime).toBe(299 * 23_543_520);
  });

  it('rounds fractional source times to the nearest flick', () => {
    const range = item(0 as Flicks, seconds(10), clip(video.id, { speed: 1 / 3 }));
    const graph = evaluate(sequence(1920, 1080, [videoTrack([range])]), assets, 1 as Flicks);
    expect(videoLayer(graph.layers).sourceTime).toBe(0);
    const later = evaluate(sequence(1920, 1080, [videoTrack([range])]), assets, 2 as Flicks);
    expect(videoLayer(later.layers).sourceTime).toBe(1);
  });

  it('fades the picture in and out linearly', () => {
    const faded = item(
      0 as Flicks,
      seconds(4),
      clip(video.id, { fadeIn: seconds(1), fadeOut: seconds(2) }),
    );
    const scene = sequence(1920, 1080, [videoTrack([faded])]);
    const opacity = (time: Flicks) =>
      videoLayer(evaluate(scene, assets, time).layers).transform.opacity;
    expect(evaluate(scene, assets, 0 as Flicks).layers).toEqual([]);
    expect(opacity(seconds(0.25))).toBeCloseTo(0.25);
    expect(opacity(seconds(1.5))).toBe(1);
    expect(opacity(seconds(3))).toBeCloseTo(0.5);
  });
});

describe('evaluate: properties and boxes', () => {
  it('resolves keyframed transforms at the item-relative time', () => {
    const animated = item(
      seconds(2),
      seconds(2),
      rectangle('#ffffff', 100, 50, {
        transform: {
          opacity: {
            kind: 'keyframed',
            keyframes: [
              { time: 0 as Flicks, value: 0.2, easing: { kind: 'linear' } },
              { time: seconds(1), value: 1, easing: { kind: 'linear' } },
            ],
          },
          rotation: 30,
        },
      }),
    );
    const layer = onlyLayer(
      evaluate(sequence(1920, 1080, [videoTrack([animated])]), assets, seconds(2.5)).layers,
    );
    expect(layer.transform.opacity).toBeCloseTo(0.6);
    expect(layer.transform.rotation).toBe(30);
    expect(layer.transform.position).toEqual({ x: 0.5, y: 0.5 });
  });

  it('fits, fills (cropped around the focus) or shows the natural size', () => {
    const vertical = (reframe: Reframe | 'fit' | 'fill' | 'custom') =>
      videoLayer(
        evaluate(
          sequence(1080, 1920, [
            videoTrack([item(0 as Flicks, seconds(1), clip(video.id, { reframe }))]),
          ]),
          assets,
          0 as Flicks,
        ).layers,
      );
    expect(vertical('fit')).toMatchObject({ box: { width: 1080, height: 607.5 }, crop: null });
    const fill = vertical('fill');
    expect(fill.box).toEqual({ width: 1080, height: 1920 });
    expect(fill.crop?.width).toBeCloseTo(0.3164, 4);
    expect(fill.crop?.x).toBeCloseTo((1 - 0.3164) / 2, 4);
    expect(vertical('custom')).toMatchObject({ box: { width: 1920, height: 1080 }, crop: null });
  });

  it('animates the focus of a filled clip with its keyframes', () => {
    const focus = {
      kind: 'keyframed' as const,
      keyframes: [
        { time: 0 as Flicks, value: { x: 0, y: 0.5 }, easing: { kind: 'linear' as const } },
        { time: seconds(1), value: { x: 1, y: 0.5 }, easing: { kind: 'linear' as const } },
      ],
    };
    const moving = item(
      0 as Flicks,
      seconds(2),
      clip(video.id, { reframe: { kind: 'fill', focus } }),
    );
    const cropAt = (time: number) =>
      videoLayer(
        evaluate(sequence(1080, 1920, [videoTrack([moving])]), assets, seconds(time)).layers,
      ).crop;
    expect(cropAt(0)?.x).toBe(0);
    expect(cropAt(0.5)?.x).toBeCloseTo(0.5 - 0.3164 / 2, 4);
    expect(cropAt(1.5)?.x).toBeCloseTo(1 - 0.3164, 4);
  });

  it('turns the display size of rotated footage upright', () => {
    const phone = videoAsset({ width: 1920, height: 1080, rotation: 90 });
    const range = item(0 as Flicks, seconds(1), clip(phone.id));
    const graph = evaluate(
      sequence(1920, 1080, [videoTrack([range])]),
      assetTable(phone),
      0 as Flicks,
    );
    expect(videoLayer(graph.layers).box).toEqual({ width: 607.5, height: 1080 });
  });

  it('fits images into the frame and scales design pixels', () => {
    const logo = imageAsset(400, 400);
    const graph = evaluate(
      sequence(3840, 2160, [
        videoTrack([item(0 as Flicks, seconds(1), image(logo.id))]),
        videoTrack([
          item(0 as Flicks, seconds(1), rectangle('#ff000080', 100, 50, { cornerRadius: 8 })),
        ]),
      ]),
      assetTable(logo),
      0 as Flicks,
    );
    expect(graph.designScale).toBe(2);
    expect(graph.layers[0]).toMatchObject({ kind: 'image', box: { width: 2160, height: 2160 } });
    expect(graph.layers[1]).toMatchObject({
      kind: 'shape',
      box: { width: 200, height: 100 },
      cornerRadius: 16,
      fill: { r: 1, g: 0, b: 0, a: 128 / 255 },
    });
  });

  it('turns texts and captions into text layers in sequence pixels', () => {
    const title = item(
      0 as Flicks,
      seconds(1),
      text('Bonjour', { style: { fontSize: still(72), color: still('#ffcc00') } }),
    );
    const caption = item(0 as Flicks, seconds(1), { kind: 'caption', text: 'Sous-titre' });
    const graph = evaluate(
      sequence(1280, 720, [videoTrack([title]), videoTrack([caption])]),
      assets,
      0 as Flicks,
    );
    expect(graph.layers[0]).toMatchObject({
      kind: 'text',
      content: 'Bonjour',
      fontSize: 48,
      font: { family: 'Inter', weight: 600, italic: false },
      color: { r: 1, g: 0.8, b: 0, a: 1 },
    });
    expect(graph.layers[1]).toMatchObject({ kind: 'text', content: 'Sous-titre' });
  });
});

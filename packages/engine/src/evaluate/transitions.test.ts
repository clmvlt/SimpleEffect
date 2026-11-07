import type { Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import {
  assetTable,
  clip,
  crossFade,
  item,
  rectangle,
  seconds,
  sequence,
  videoAsset,
  videoTrack,
} from '../testing/document';
import { evaluate } from './evaluate';
import type { Layer, VideoLayer } from './frame-graph';
import { fadeFactor } from './timing';
import { playExtensions, trackTransitionSpans, transitionExtent } from './transitions';

const video = videoAsset({ width: 1920, height: 1080, duration: seconds(60) });
const assets = assetTable(video);

/** Clip A `[0, 4 s)` from 10 s, clip B `[4 s, 8 s)` from 20 s, a centred 2 s dissolve at 4 s. */
function dissolveScene(options: Parameters<typeof crossFade>[3] = {}) {
  const outgoing = item(0 as Flicks, seconds(4), clip(video.id, { sourceStart: seconds(10) }));
  const incoming = item(seconds(4), seconds(4), clip(video.id, { sourceStart: seconds(20) }));
  const transition = crossFade(outgoing, incoming, seconds(2), options);
  const scene = sequence(1920, 1080, [videoTrack([outgoing, incoming], false, [transition])]);
  return { outgoing, incoming, transition, scene };
}

function asVideo(layer: Layer | undefined): VideoLayer {
  if (layer?.kind !== 'video') throw new Error('expected a video layer');
  return layer;
}

describe('transition spans', () => {
  it('split the duration around the cut like the domain', () => {
    expect(transitionExtent({ duration: 11 as Flicks, alignment: 'center' })).toEqual({
      before: 5,
      after: 6,
    });
    expect(transitionExtent({ duration: 10 as Flicks, alignment: 'start' })).toEqual({
      before: 0,
      after: 10,
    });
    expect(transitionExtent({ duration: 10 as Flicks, alignment: 'end' })).toEqual({
      before: 10,
      after: 0,
    });
  });

  it('extend the items they join and ignore transitions off a cut', () => {
    const { outgoing, incoming, transition } = dissolveScene();
    const spans = trackTransitionSpans([outgoing, incoming], [transition]);
    expect(spans).toEqual([
      expect.objectContaining({ start: seconds(3), cut: seconds(4), end: seconds(5) }),
    ]);
    const extensions = playExtensions(spans);
    expect(extensions.get(outgoing.id)).toEqual({ before: 0, after: seconds(1) });
    expect(extensions.get(incoming.id)).toEqual({ before: seconds(1), after: 0 });
    const apart = { ...incoming, timelineRange: { start: seconds(5), duration: seconds(3) } };
    expect(trackTransitionSpans([outgoing, apart], [transition])).toEqual([]);
  });
});

describe('evaluate: cross dissolves', () => {
  it('draw both clips as one dissolve group weighted by progress', () => {
    const { outgoing, incoming, transition, scene } = dissolveScene();
    const layers = evaluate(scene, assets, seconds(4.5)).layers;
    expect(layers).toHaveLength(2);
    const [out, into] = [asVideo(layers[0]), asVideo(layers[1])];
    expect(out.itemId).toBe(outgoing.id);
    expect(into.itemId).toBe(incoming.id);
    expect(out.dissolve).toEqual({ transitionId: transition.id, side: 'outgoing' });
    expect(into.dissolve).toEqual({ transitionId: transition.id, side: 'incoming' });
    expect(out.transform.opacity).toBeCloseTo(0.25, 9);
    expect(into.transform.opacity).toBeCloseTo(0.75, 9);
    // The outgoing clip plays its media past its end, the incoming one before its start.
    expect(out.sourceTime).toBe(seconds(14.5));
    expect(asVideo(evaluate(scene, assets, seconds(3.5)).layers[1]).sourceTime).toBe(seconds(19.5));
  });

  it('show a single clip outside the span', () => {
    const { outgoing, incoming, scene } = dissolveScene();
    const before = evaluate(scene, assets, seconds(2.9)).layers;
    expect(before.map((layer) => layer.itemId)).toEqual([outgoing.id]);
    expect(before[0]?.dissolve).toBeUndefined();
    const after = evaluate(scene, assets, seconds(5)).layers;
    expect(after.map((layer) => layer.itemId)).toEqual([incoming.id]);
  });

  it('follow the alignment and leave invisible sides out', () => {
    const { incoming, scene } = dissolveScene({ alignment: 'start' });
    // Starting at the cut: nothing before it, and at the cut the incoming weight is 0.
    expect(evaluate(scene, assets, seconds(3.9)).layers).toHaveLength(1);
    const atCut = evaluate(scene, assets, seconds(4)).layers;
    expect(atCut).toHaveLength(1);
    expect(atCut[0]?.dissolve?.side).toBe('outgoing');
    expect(evaluate(scene, assets, seconds(5)).layers.map((layer) => layer.itemId)).toEqual([
      expect.any(String),
      incoming.id,
    ]);
  });

  it('blend any items, not only clips, and ignore sound cross-fades', () => {
    const red = item(0 as Flicks, seconds(2), rectangle('#ff0000', 100, 100));
    const blue = item(seconds(2), seconds(2), rectangle('#0000ff', 100, 100));
    const shapes = sequence(1920, 1080, [
      videoTrack([red, blue], false, [crossFade(red, blue, seconds(1))]),
    ]);
    expect(evaluate(shapes, assets, seconds(2)).layers).toHaveLength(2);
    const sound = crossFade(red, blue, seconds(1), { kind: 'audioCrossfade' });
    const silent = sequence(1920, 1080, [videoTrack([red, blue], false, [sound])]);
    expect(evaluate(silent, assets, seconds(2)).layers).toHaveLength(1);
  });
});

describe('fades', () => {
  const faded = {
    fadeIn: seconds(1),
    fadeOut: seconds(2),
    fadeInShape: 'constantPower',
    fadeOutShape: 'linear',
  } as const;

  it('follow their shapes and hold at the edges', () => {
    expect(fadeFactor(faded, seconds(10), seconds(0.5))).toBeCloseTo(Math.SQRT1_2, 9);
    expect(fadeFactor(faded, seconds(10), seconds(9))).toBeCloseTo(0.5, 9);
    expect(fadeFactor(faded, seconds(10), seconds(5))).toBe(1);
    expect(fadeFactor(faded, seconds(10), seconds(-1))).toBe(0);
    expect(fadeFactor(faded, seconds(10), seconds(11))).toBe(0);
  });
});

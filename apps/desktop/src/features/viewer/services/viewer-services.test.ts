import type { Asset, CoreEndpoint, Flicks, Transition } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import {
  captionItem,
  sequence,
  videoTrack,
} from '../../../shared/project-store/testing/session-fixture';
import { DEFAULT_VIEWER_PREFERENCES, parseViewerPreferences } from '../store/viewer-store';
import { summarizeIssues } from './issue-summary';
import { createMediaLocator } from './media-locator';
import { sceneOf } from './viewer-scene';
import { FIT_MARGIN, viewerLayout } from './viewer-layout';

const HD = { width: 1920, height: 1080 };

describe('viewerLayout', () => {
  it('fits the sequence ratio into the stage, minus a margin', () => {
    const layout = viewerLayout({ width: 976 + 2 * FIT_MARGIN, height: 900 }, HD, 'fit', 1);
    expect(layout.display).toEqual({ width: 976, height: 549 });
    expect(layout.pixels).toEqual({ width: 976, height: 549 });
  });

  it('shows fixed zooms at their scale, whatever the stage', () => {
    expect(viewerLayout({ width: 100, height: 100 }, HD, 0.5, 1).display).toEqual({
      width: 960,
      height: 540,
    });
    expect(viewerLayout({ width: 100, height: 100 }, HD, 2, 1).display).toEqual({
      width: 3840,
      height: 2160,
    });
  });

  it('draws device pixels but never more than the sequence resolution', () => {
    expect(viewerLayout({ width: 0, height: 0 }, HD, 0.5, 2).pixels).toEqual(HD);
    expect(viewerLayout({ width: 0, height: 0 }, HD, 0.5, 1).pixels).toEqual({
      width: 960,
      height: 540,
    });
    expect(viewerLayout({ width: 0, height: 0 }, HD, 'fit', 2).pixels).toEqual({
      width: 1,
      height: 1,
    });
  });
});

describe('sceneOf', () => {
  it('lists the tracks in stacking order with their items from the mirror', () => {
    const scene = sceneOf({
      sequence: sequence(['v1', 'v2', 'gone']),
      tracks: {
        v1: videoTrack('v1', ['a', 'b']),
        v2: { ...videoTrack('v2', []), hidden: true },
      },
      items: { a: captionItem('a', 'v1', 0), b: captionItem('b', 'v1', 5000) },
      assets: {},
    });
    expect(scene.sequence.format).toMatchObject(HD);
    expect(scene.sequence.tracks.map((track) => [track.hidden, track.items.length])).toEqual([
      [false, 2],
      [true, 0],
    ]);
  });

  it('keeps the transitions of each track for the engines', () => {
    const dissolve: Transition = {
      id: 't1',
      kind: 'crossDissolve',
      leftItem: 'a',
      rightItem: 'b',
      duration: 1000 as Flicks,
      alignment: 'center',
    };
    const scene = sceneOf({
      sequence: sequence(['v1']),
      tracks: { v1: { ...videoTrack('v1', ['a', 'b']), transitions: [dissolve] } },
      items: { a: captionItem('a', 'v1', 0), b: captionItem('b', 'v1', 5000) },
      assets: {},
    });
    expect(scene.sequence.tracks[0]?.transitions).toEqual([dissolve]);
  });
});

describe('createMediaLocator', () => {
  const endpoint: CoreEndpoint = { url: 'http://127.0.0.1:4000', token: 'secret' };
  const asset = {
    id: 'asset-1',
    fingerprint: { sizeBytes: 1, modifiedUnixMs: 1, sampleHash: 'abc' },
    derived: { mezzanine: null, proxy: { relativePath: 'cache/p.mp4', status: { kind: 'ready' } } },
  } as unknown as Asset;

  it('builds versioned /media URLs with the token', () => {
    const locate = createMediaLocator(endpoint, { [asset.id]: asset });
    expect(locate(asset.id, 'original')).toBe(
      'http://127.0.0.1:4000/media/assets/asset-1/original?t=secret&v=abc',
    );
    expect(locate(asset.id, 'proxy')).toBe(
      'http://127.0.0.1:4000/media/assets/asset-1/proxy?t=secret&v=abc%3Acache%2Fp.mp4',
    );
  });

  it('has no URL before the core is known or for an unknown asset', () => {
    expect(createMediaLocator(null, { [asset.id]: asset })(asset.id, 'original')).toBeNull();
    expect(createMediaLocator(endpoint, {})('other', 'original')).toBeNull();
  });
});

describe('summarizeIssues', () => {
  it('reports the most severe kind and its count', () => {
    expect(summarizeIssues([])).toBeNull();
    expect(
      summarizeIssues([
        { assetId: 'a', kind: 'preparing' },
        { assetId: 'b', kind: 'missing' },
        { assetId: 'c', kind: 'missing' },
      ]),
    ).toEqual({ kind: 'missing', count: 2 });
  });
});

describe('parseViewerPreferences', () => {
  it('reads saved preferences and ignores malformed values', () => {
    expect(
      parseViewerPreferences('{"zoom":2,"proxy":true,"checkerboard":true,"loop":true}'),
    ).toEqual({
      ...DEFAULT_VIEWER_PREFERENCES,
      zoom: 2,
      proxy: true,
      checkerboard: true,
      loop: true,
    });
    expect(parseViewerPreferences('{"zoom":3,"proxy":"yes"}')).toEqual(DEFAULT_VIEWER_PREFERENCES);
    expect(parseViewerPreferences('not json')).toEqual(DEFAULT_VIEWER_PREFERENCES);
    expect(parseViewerPreferences(null)).toEqual(DEFAULT_VIEWER_PREFERENCES);
  });
});

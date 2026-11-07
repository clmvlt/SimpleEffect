import { describe, expect, it } from 'vitest';

import { seconds, testAsset, testItem, testTrack } from '../testing/fixtures';
import { clipboardOf, pastedItems } from './clipboard';
import { DEFAULT_IMAGE_DURATION, assetParts, dropGhosts, dropTargets } from './drop-placement';
import {
  canLink,
  canUnlink,
  duplicatePlacements,
  itemsToSplit,
  nudgePlacements,
} from './item-edits';

const tracks = [testTrack('v1', 'video'), testTrack('v2', 'video'), testTrack('a1', 'audio')];

describe('drop placement', () => {
  it('sends sound to the audio track with the same number, or a new one', () => {
    const [v1, v2, a1] = tracks;
    if (v1 === undefined || v2 === undefined || a1 === undefined) throw new Error('tracks');
    expect(dropTargets(tracks, v1)).toEqual({
      video: { kind: 'existing', trackId: 'v1' },
      audio: { kind: 'existing', trackId: 'a1' },
    });
    expect(dropTargets(tracks, v2).audio).toEqual({ kind: 'new' });
    expect(dropTargets(tracks, a1).video).toEqual({ kind: 'existing', trackId: 'v1' });
    expect(dropTargets(tracks, null)).toEqual({
      video: { kind: 'existing', trackId: 'v2' },
      audio: { kind: 'new' },
    });
  });

  it('places assets one after the other, a video with sound on two rows', () => {
    const video = testAsset('clip', '/media/clip.mp4');
    const image = testAsset('logo', '/media/logo.png', 'image');
    expect(assetParts(image)).toEqual({
      video: true,
      audio: false,
      duration: DEFAULT_IMAGE_DURATION,
    });
    const ghosts = dropGhosts([video, image], seconds(2), {
      video: { kind: 'existing', trackId: 'v1' },
      audio: { kind: 'new' },
    });
    expect(ghosts.map((ghost) => [ghost.row, ghost.start])).toEqual([
      [{ kind: 'track', trackId: 'v1' }, seconds(2)],
      [{ kind: 'newAudio' }, seconds(2)],
      [{ kind: 'track', trackId: 'v1' }, seconds(32)],
    ]);
  });
});

describe('clipboard', () => {
  const a = testItem('a', 'v1', 4, 1);
  const s = testItem('s', 'a1', 5, 1);

  it('pastes at the playhead, the active track receiving the lowest copied track of its kind', () => {
    const clipboard = clipboardOf('project-1', [a, s], tracks);
    if (clipboard === null) throw new Error('copied');
    const [, v2] = tracks;
    const pasted = pastedItems(clipboard, tracks, seconds(10), v2 ?? null);
    expect(pasted.map((entry) => [entry.trackId, entry.item.timelineRange.start])).toEqual([
      ['v2', seconds(10)],
      ['a1', seconds(11)],
    ]);
    expect(pasted[0]?.item).not.toHaveProperty('trackId');
  });

  it('keeps every copy on an existing track', () => {
    const top = testItem('t', 'v2', 0, 1);
    const clipboard = clipboardOf('project-1', [top], tracks);
    if (clipboard === null) throw new Error('copied');
    expect(pastedItems(clipboard, tracks, seconds(0), tracks[0] ?? null)[0]?.trackId).toBe('v1');
    expect(clipboardOf('project-1', [], tracks)).toBeNull();
  });
});

describe('item edits', () => {
  const a = testItem('a', 'v1', 0, 2, { linkGroup: 'g' });
  const b = testItem('b', 'v2', 1, 2, { linkGroup: 'g' });
  const c = testItem('c', 'a1', 1, 4);

  it('cuts the selection, or everything unlocked under the playhead', () => {
    expect(itemsToSplit([a, b, c], new Set(), new Set(['a1']), seconds(1.5))).toEqual(['a', 'b']);
    expect(itemsToSplit([a, b, c], new Set(['c']), new Set(), seconds(1.5))).toEqual(['c']);
    expect(itemsToSplit([a, b, c], new Set(), new Set(), seconds(1))).toEqual(['a']);
  });

  it('nudges without going before zero and duplicates after the block', () => {
    expect(nudgePlacements([a], seconds(-1))).toBeNull();
    expect(nudgePlacements([b, c], seconds(-2))?.map((placement) => placement.start)).toEqual([
      0, 0,
    ]);
    expect(duplicatePlacements([a, b]).map((placement) => placement.start)).toEqual([
      seconds(3),
      seconds(4),
    ]);
  });

  it('links items not already in one group, unlinks linked ones', () => {
    expect(canLink([a, b])).toBe(false);
    expect(canLink([a, c])).toBe(true);
    expect(canLink([c])).toBe(false);
    expect(canUnlink([c])).toBe(false);
    expect(canUnlink([a, c])).toBe(true);
  });
});

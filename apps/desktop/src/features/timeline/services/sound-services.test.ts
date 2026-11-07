import type { Animatable, Flicks, ItemEntity, TimeRange, TrackEntity } from '@se/protocol';
import type { TransitionSpan } from '@se/engine';
import { describe, expect, it } from 'vitest';

import { parseEditPreferences } from '../store/edit-preferences-store';
import { seconds, testItem, testTrack } from '../testing/fixtures';
import {
  draggedTransitionDuration,
  fadeLength,
  openCuts,
  silencesInItem,
  transitionsAtPlayhead,
} from './audio-edits';
import {
  SILENCE_DB,
  addVolumeKeyframe,
  formatVolume,
  fractionToVolume,
  moveVolumeKeyframe,
  removeVolumeKeyframe,
  shiftVolume,
  volumeLinePoints,
  volumeToFraction,
} from './volume-line';

const RATE = { numerator: 25, denominator: 1 };
const LINEAR = { kind: 'linear' } as const;

function clipItem(id: string, start: number, duration: number, fades = { in: 0, out: 0 }) {
  return testItem(id, 'a1', start, duration, {
    kind: {
      kind: 'mediaClip',
      assetId: 'asset',
      sourceStart: seconds(10),
      speed: 2,
      transform: null as never,
      volumeDb: { kind: 'static', value: 0 },
      fadeIn: seconds(fades.in),
      fadeOut: seconds(fades.out),
      fadeInShape: 'linear',
      fadeOutShape: 'linear',
      reframe: { kind: 'fit' },
    },
  });
}

describe('volume line', () => {
  it('maps decibels to heights and back, silence at the bottom', () => {
    expect(volumeToFraction(6)).toBe(1);
    expect(volumeToFraction(-60)).toBe(0);
    expect(volumeToFraction(-90)).toBe(0);
    expect(fractionToVolume(volumeToFraction(-12))).toBe(-12);
    expect(fractionToVolume(0)).toBe(SILENCE_DB);
    expect(formatVolume(-6.26)).toBe('−6,3 dB');
    expect(formatVolume(SILENCE_DB)).toBe('−∞ dB');
  });

  it('draws the held value around the keyframes', () => {
    const curve: Animatable<number> = {
      kind: 'keyframed',
      keyframes: [{ time: seconds(1), value: -6, easing: LINEAR }],
    };
    expect(volumeLinePoints(curve, seconds(2))).toEqual([
      { time: 0, db: -6 },
      { time: seconds(1), db: -6 },
      { time: seconds(2), db: -6 },
    ]);
  });

  it('adds, moves, removes keyframes and shifts the whole curve', () => {
    const still: Animatable<number> = { kind: 'static', value: -3 };
    const one = addVolumeKeyframe(still, seconds(1));
    expect(one).toEqual({
      kind: 'keyframed',
      keyframes: [{ time: seconds(1), value: -3, easing: LINEAR }],
    });
    const two = addVolumeKeyframe(one, 0 as Flicks);
    const moved = moveVolumeKeyframe(two, 0, seconds(5), -12, seconds(4));
    // Kept before its neighbour at 1 s.
    expect(moved.kind === 'keyframed' && moved.keyframes[0]).toEqual({
      time: seconds(1) - 1,
      value: -12,
      easing: LINEAR,
    });
    expect(removeVolumeKeyframe(removeVolumeKeyframe(two, 0), 0)).toEqual({
      kind: 'static',
      value: -3,
    });
    expect(shiftVolume(still, 30)).toEqual({ kind: 'static', value: 24 });
  });
});

describe('fades and transitions', () => {
  it('sizes fades in whole frames without overlapping the other one', () => {
    const item = clipItem('m', 0, 4, { in: 0, out: 1 });
    expect(fadeLength(item, 'in', seconds(0.5), RATE)).toBe(seconds(0.52));
    expect(fadeLength(item, 'in', seconds(9), RATE)).toBe(seconds(3));
    expect(fadeLength(item, 'out', seconds(0.4), RATE)).toBe(seconds(0.6));
    expect(fadeLength(item, 'out', seconds(-9), RATE)).toBe(seconds(4));
  });

  const tracks: TrackEntity[] = [
    testTrack('v1', 'video', ['a', 'b']),
    testTrack('a1', 'audio', ['c', 'd'], { locked: true }),
  ];
  const itemsByTrack = new Map<string, readonly ItemEntity[]>([
    ['v1', [testItem('a', 'v1', 0, 2), testItem('b', 'v1', 2, 2)]],
    ['a1', [testItem('c', 'a1', 0, 2), testItem('d', 'a1', 2, 2)]],
  ]);

  it('finds the cut nearest the playhead on unlocked tracks', () => {
    const wanted = transitionsAtPlayhead(tracks, itemsByTrack, seconds(2.4), seconds(1), new Set());
    expect(wanted).toEqual([
      {
        trackId: 'v1',
        leftItemId: 'a',
        rightItemId: 'b',
        duration: seconds(1),
        alignment: 'center',
      },
    ]);
    expect(transitionsAtPlayhead(tracks, itemsByTrack, seconds(5), seconds(1), new Set())).toEqual(
      [],
    );
    expect(openCuts(tracks[0] as TrackEntity, itemsByTrack.get('v1') ?? [])).toHaveLength(1);
  });

  it('resizes from a dragged edge', () => {
    const centred: TransitionSpan = {
      id: 't',
      leftItem: 'a',
      rightItem: 'b',
      start: seconds(1.5),
      cut: seconds(2),
      end: seconds(2.5),
    };
    expect(draggedTransitionDuration(centred, 'center', 'end', seconds(3), RATE)).toBe(seconds(2));
    // Never shorter than one frame.
    expect(draggedTransitionDuration(centred, 'start', 'end', seconds(1.5), RATE)).toBe(
      seconds(0.04),
    );
  });

  it('places silences of the source on the clip', () => {
    // The clip plays source 10 s → 18 s at 2× over 4 s.
    const silences: TimeRange[] = [
      { start: seconds(8), duration: seconds(3) },
      { start: seconds(14), duration: seconds(1) },
      { start: seconds(30), duration: seconds(1) },
    ];
    expect(silencesInItem(clipItem('m', 0, 4), silences)).toEqual([
      { start: 0, end: seconds(0.5) },
      { start: seconds(2), end: seconds(2.5) },
    ]);
  });
});

describe('edit preferences', () => {
  it('read back saved values and fall back on anything malformed', () => {
    expect(parseEditPreferences('{"transitionSeconds":2,"showSilences":false}')).toEqual({
      transitionSeconds: 2,
      showSilences: false,
    });
    expect(parseEditPreferences('{"transitionSeconds":99}')).toEqual({
      transitionSeconds: 1,
      showSilences: true,
    });
    expect(parseEditPreferences('nope')).toEqual({ transitionSeconds: 1, showSilences: true });
  });
});

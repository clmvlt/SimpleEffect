import type { Animatable, Flicks, ItemEntity, SubjectDetection, Vec2 } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { SECOND, graphicItem } from '../../../shared/project-store/testing/graphics-fixture';
import { clipKind, fill } from '../../../shared/project-store/testing/reframe-fixture';
import { facesAt, focusKeyframes, focusTrajectory } from './subject-path';
import {
  DEFAULT_TRACKING_SETTINGS,
  reportNotice,
  shownTracking,
  trackingOf,
  withSetting,
} from './tracking';

const tracked = (smoothing: number): ItemEntity =>
  graphicItem(
    `tracked-${String(smoothing)}`,
    clipKind({
      kind: 'tracked',
      focus: { kind: 'static', value: { x: 0.3, y: 0.5 } },
      tracking: { ...DEFAULT_TRACKING_SETTINGS, smoothing },
    }),
  );

describe('tracking settings', () => {
  it('shows the settings of the first tracked clip, else the defaults', () => {
    const filled = graphicItem('filled', clipKind(fill()));
    expect(trackingOf(filled)).toBeNull();
    expect(shownTracking([filled])).toEqual(DEFAULT_TRACKING_SETTINGS);
    expect(shownTracking([filled, tracked(0.8), tracked(0.2)]).smoothing).toBe(0.8);
  });

  it('keeps edited values in their range, to the hundredth', () => {
    expect(withSetting(DEFAULT_TRACKING_SETTINGS, 'smoothing', 1.4).smoothing).toBe(1);
    expect(withSetting(DEFAULT_TRACKING_SETTINGS, 'eyeLine', 0.05).eyeLine).toBe(0.2);
    expect(withSetting(DEFAULT_TRACKING_SETTINGS, 'deadZone', 0.123).deadZone).toBe(0.12);
  });

  it('tells what a request did', () => {
    expect(reportNotice({ appliedItemIds: ['a', 'b'], waitingAssetIds: [], skipped: [] })).toEqual({
      kind: 'applied',
      count: 2,
    });
    expect(reportNotice({ appliedItemIds: [], waitingAssetIds: ['asset'], skipped: [] })).toEqual({
      kind: 'analysing',
      count: 1,
    });
    expect(
      reportNotice({
        appliedItemIds: [],
        waitingAssetIds: [],
        skipped: [{ itemId: 'title', reason: 'notAVideoClip' }],
      }),
    ).toEqual({ kind: 'nothing' });
  });
});

const ms = (value: number) => ((value * SECOND) / 1000) as Flicks;

const detection: SubjectDetection = {
  model: 'yunet-2026may',
  interval: ms(200),
  frames: [0, 200, 400, 600].map((time) => ({
    time: ms(time),
    faces: time === 400 ? [] : [{ x: time / 1000, y: 0.2, width: 0.1, height: 0.14, score: 0.9 }],
  })),
};

describe('the subject seen in the viewer', () => {
  it('finds the faces of the nearest sample within one interval', () => {
    expect(facesAt(detection, ms(190))[0]?.x).toBe(0.2);
    expect(facesAt(detection, ms(90))[0]?.x).toBe(0);
    expect(facesAt(detection, ms(410))).toEqual([]);
    expect(facesAt(detection, ms(900))).toEqual([]);
  });

  it('samples the focus along the clip as the engine interpolates it', () => {
    const focus: Animatable<Vec2> = {
      kind: 'keyframed',
      keyframes: [
        { time: 0 as Flicks, value: { x: 0.2, y: 0.5 }, easing: { kind: 'linear' } },
        { time: (2 * SECOND) as Flicks, value: { x: 0.6, y: 0.5 }, easing: { kind: 'linear' } },
      ],
    };
    const path = focusTrajectory(focus, (2 * SECOND) as Flicks, 4);
    expect(path.map((point) => Math.round(point.x * 100) / 100)).toEqual([0.2, 0.3, 0.4, 0.5, 0.6]);
    expect(focusKeyframes(focus)).toHaveLength(2);
    expect(focusKeyframes({ kind: 'static', value: { x: 0.5, y: 0.5 } })).toEqual([]);
  });
});

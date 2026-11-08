import type { Asset, Flicks, ItemEntity, TimeRange, TranscriptionStatus } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { task } from '../../../shared/ai-tasks/testing/task-fixture';
import { mirrorFromSession } from '../../../shared/project-store';
import { transform } from '../../../shared/project-store/testing/graphics-fixture';
import { session } from '../../../shared/project-store/testing/session-fixture';
import {
  CONTINUE_WINDOW_MS,
  continuableTask,
  smallSelection,
  speechReadiness,
  untranscribedAssets,
} from './composer-notices';
import type { DerivedScope } from './task-scope';

const SECOND = 705_600_000;
const range = (start: number, end: number): TimeRange => ({
  start: (start * SECOND) as Flicks,
  duration: ((end - start) * SECOND) as Flicks,
});

function derived(source: DerivedScope['rangeSource'], scopeRange: TimeRange | null): DerivedScope {
  return {
    scope: { sequenceId: 'sequence-1', range: scopeRange, tracks: null },
    rangeSource: source,
    tracksSource: 'all',
  };
}

describe('smallSelection', () => {
  it('points out a selection covering less than a tenth of the sequence', () => {
    const tiny = range(0, 0.133);
    expect(smallSelection(derived('selection', tiny), 69 * SECOND)).toEqual(tiny);
    expect(smallSelection(derived('selection', range(0, 20)), 69 * SECOND)).toBeNull();
  });

  it('leaves other sources alone', () => {
    expect(smallSelection(derived('inOut', range(0, 1)), 69 * SECOND)).toBeNull();
    expect(smallSelection(derived('text', range(0, 1)), 69 * SECOND)).toBeNull();
    expect(smallSelection(derived('sequence', null), 69 * SECOND)).toBeNull();
  });
});

function soundAsset(id: string, transcribed: boolean): Asset {
  return {
    id,
    originalPath: `/rushes/${id}.mp4`,
    relativeHint: null,
    kind: 'video',
    mediaInfo: null,
    fingerprint: null,
    derived: { mezzanine: null, proxy: null },
    analysis: {
      audio: null,
      scenes: null,
      transcript: transcribed
        ? { language: 'fr', model: 'base', wordCount: 4, transcribedAt: 7 }
        : null,
    },
    decodability: { video: 'decodable', audio: 'decodable' },
    missing: false,
  };
}

function soundClip(id: string, assetId: string, start: number): ItemEntity {
  return {
    id,
    trackId: 'a1',
    revision: 1,
    origin: { kind: 'user' },
    timelineRange: range(start, start + 5),
    linkGroup: null,
    kind: {
      kind: 'mediaClip',
      assetId,
      sourceStart: 0 as Flicks,
      speed: 1,
      transform: transform(),
      volumeDb: { kind: 'static', value: 0 },
      fadeIn: 0 as Flicks,
      fadeOut: 0 as Flicks,
      fadeInShape: 'linear',
      fadeOutShape: 'linear',
      reframe: { kind: 'fit' },
    },
  };
}

describe('untranscribedAssets', () => {
  it('lists the sound media of the sequence without a transcript, once each', () => {
    const base = session();
    const sequence = base.snapshot.sequences[0];
    if (sequence === undefined) throw new Error('the fixture has a sequence');
    const mirror = mirrorFromSession({
      ...base,
      snapshot: {
        ...base.snapshot,
        sequences: [{ ...sequence, trackIds: ['a1'] }],
        tracks: [
          {
            id: 'a1',
            sequenceId: sequence.id,
            kind: 'audio',
            name: 'A1',
            muted: false,
            solo: false,
            locked: false,
            hidden: false,
            volumeDb: 0,
            pan: 0,
            itemIds: ['clip-1', 'clip-2', 'clip-3'],
            transitions: [],
          },
        ],
        items: [
          soundClip('clip-1', 'interview', 0),
          soundClip('clip-2', 'voice', 5),
          soundClip('clip-3', 'voice', 10),
        ],
        assets: [soundAsset('interview', true), soundAsset('voice', false)],
      },
    });
    expect(untranscribedAssets(mirror, sequence.id)).toEqual(['voice']);
  });
});

describe('speechReadiness', () => {
  const status = (installed: boolean, problem: string | null) =>
    ({
      whisper: { path: null, problem },
      models: [{ installed }],
    }) as unknown as TranscriptionStatus;

  it('needs whisper.cpp and an installed model', () => {
    expect(speechReadiness(status(true, null))).toBe('ready');
    expect(speechReadiness(status(false, null))).toBe('noModel');
    expect(speechReadiness(status(true, 'whisper-cli not found'))).toBe('unavailable');
  });
});

describe('continuableTask', () => {
  const finished = (id: string, finishedAt: number, overrides = {}) =>
    task({
      id,
      status: { kind: 'completed' },
      agentSessionId: `session-${id}`,
      runs: [
        {
          runKind: 'initial',
          prompt: 'Coupe les blancs',
          status: { kind: 'completed' },
          queuedAt: finishedAt - 1000,
          startedAt: finishedAt - 900,
          finishedAt,
        },
      ],
      ...overrides,
    });
  const now = 10 * CONTINUE_WINDOW_MS;

  it('offers the latest task that finished recently on the same sequence', () => {
    const older = finished('task-1', now - 2 * CONTINUE_WINDOW_MS);
    const recent = finished('task-2', now - 60_000);
    expect(continuableTask([older, recent], 'sequence-1', now)?.id).toBe('task-2');
  });

  it('offers nothing when the latest task is old, running, sessionless or elsewhere', () => {
    expect(continuableTask([finished('a', now - CONTINUE_WINDOW_MS - 1)], 'sequence-1', now)).toBe(
      null,
    );
    const running = finished('b', now, { status: { kind: 'running' } });
    expect(continuableTask([finished('a', now - 1000), running], 'sequence-1', now)).toBeNull();
    expect(
      continuableTask([finished('c', now, { agentSessionId: null })], 'sequence-1', now),
    ).toBeNull();
    expect(continuableTask([finished('d', now)], 'sequence-2', now)).toBeNull();
    expect(continuableTask([], 'sequence-1', now)).toBeNull();
  });
});

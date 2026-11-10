import type {
  Asset,
  ExportInfo,
  Flicks,
  FontFamily,
  Format,
  ItemEntity,
  ItemKind,
} from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { textKind } from '../../../shared/project-store/testing/graphics-fixture';
import {
  VIDEO_ASSET,
  clipKind,
  fill,
  videoAsset,
} from '../../../shared/project-store/testing/reframe-fixture';
import { FLICKS_PER_SECOND } from '../../../shared/time';
import {
  exportIdOfTag,
  exportNotice,
  exportNotificationTag,
  fileNameOf,
  renderIssueCount,
} from './export-notice';
import { contentEnd, exportPlan, type ExportPlanInput } from './export-plan';
import { exportWarnings, isBlocked, type WarningInput } from './export-warnings';

const HD: Format = {
  width: 1920,
  height: 1080,
  frameRate: { numerator: 25, denominator: 1 },
  sampleRate: 48_000,
};
const seconds = (value: number) => (value * FLICKS_PER_SECOND) as Flicks;

function item(id: string, kind: ItemKind, start: number, duration: number): ItemEntity {
  return {
    trackId: 'track-v1',
    id,
    revision: 1,
    origin: { kind: 'user' },
    timelineRange: { start: seconds(start), duration: seconds(duration) },
    linkGroup: null,
    kind,
  };
}

/** The fixture's clip, of the asset `assetId`. */
function clipOf(assetId: string): ItemKind {
  const kind = clipKind(fill());
  if (kind.kind !== 'mediaClip') throw new Error('the fixture is a media clip');
  return { ...kind, assetId };
}

/** The fixture's text, in the font `family`. */
function textIn(family: string): ItemKind {
  const kind = textKind();
  if (kind.kind !== 'text') throw new Error('the fixture is a text');
  return { ...kind, style: { ...kind.style, fontFamily: family } };
}

const INTER: FontFamily = { name: 'Inter', source: 'bundled', faces: [] };
const missingAsset: Asset = { ...videoAsset(), missing: true };

function input(overrides: Partial<WarningInput> = {}): WarningInput {
  return {
    items: [item('clip', clipKind(fill()), 0, 10)],
    assets: { [VIDEO_ASSET]: videoAsset() },
    captionStyles: [],
    families: [INTER],
    format: HD,
    start: seconds(0),
    end: seconds(10),
    aspect: null,
    support: { video: 'hardware', audio: 'webCodecs' },
    aacByFfmpeg: false,
    ...overrides,
  };
}

describe('exportWarnings', () => {
  it('has nothing to say about a ready export', () => {
    expect(exportWarnings(input())).toEqual([]);
  });

  it('blocks empty ranges and missing encoders, listed first', () => {
    const warnings = exportWarnings(
      input({
        start: seconds(5),
        end: seconds(5),
        aspect: { width: 9, height: 16 },
        support: { video: 'none', audio: 'none' },
      }),
    );
    expect(warnings.map((warning) => warning.kind)).toEqual([
      'emptyRange',
      'noVideoEncoder',
      'noAudioEncoder',
      'aspect',
    ]);
    expect(isBlocked(warnings)).toBe(true);
  });

  it('informs about software encoding and AAC handed to ffmpeg', () => {
    const warnings = exportWarnings(
      input({ support: { video: 'software', audio: 'none' }, aacByFfmpeg: true }),
    );
    expect(warnings.map((warning) => warning.kind)).toEqual(['softwareEncoding', 'aacByFfmpeg']);
    expect(isBlocked(warnings)).toBe(false);
  });

  it('counts missing and preparing media inside the range only', () => {
    const preparing: Asset = {
      ...videoAsset('asset-2'),
      decodability: { video: 'unknown', audio: 'noStream' },
    };
    const otherClip = clipOf('asset-2');
    const warnings = exportWarnings(
      input({
        items: [item('clip', clipKind(fill()), 0, 4), item('later', otherClip, 20, 4)],
        assets: { [VIDEO_ASSET]: missingAsset, 'asset-2': preparing },
      }),
    );
    expect(warnings).toEqual([{ kind: 'missingMedia', count: 1 }]);
    expect(
      exportWarnings(
        input({ items: [item('later', otherClip, 2, 4)], assets: { 'asset-2': preparing } }),
      ),
    ).toEqual([{ kind: 'preparingMedia', count: 1 }]);
  });

  it('names fonts the catalogue lacks once it is listed', () => {
    const items = [item('title', textIn('Gone'), 1, 2), item('plain', textIn('Inter'), 1, 2)];
    expect(exportWarnings(input({ items }))).toEqual([
      { kind: 'missingFonts', families: ['Gone'] },
    ]);
    expect(exportWarnings(input({ items, families: null }))).toEqual([]);
  });
});

describe('exportPlan', () => {
  const planInput = (overrides: Partial<ExportPlanInput> = {}): ExportPlanInput => ({
    draft: {
      container: 'mp4',
      video: { codec: 'h264', shortSide: 720, rate: { kind: 'bitrate', bitsPerSecond: 8e6 } },
      audio: { codec: 'aac', bitsPerSecond: 192_000 },
    },
    format: HD,
    span: { start: seconds(0), end: seconds(10) },
    template: '{sequence} {width}x{height}',
    names: { project: 'Film', sequence: 'Montage', preset: 'YouTube' },
    now: new Date(2026, 10, 10),
    support: { video: 'hardware', audio: 'webCodecs' },
    aspect: null,
    items: [],
    assets: {},
    captionStyles: [],
    families: null,
    ...overrides,
  });

  it('gives the settings, file name, duration and estimate', () => {
    const plan = exportPlan(planInput());
    expect(plan.fileName).toBe('Montage 1280x720.mp4');
    expect(plan.seconds).toBe(10);
    expect(plan.settings.video).toMatchObject({ width: 1280, height: 720 });
    expect(plan.estimatedBytes).toBeGreaterThan(10_000_000);
    expect(plan.blocked).toBe(false);
    expect(plan.aacByFfmpeg).toBe(false);
  });

  it('hands AAC to ffmpeg when WebCodecs cannot encode it', () => {
    const plan = exportPlan(planInput({ support: { video: 'hardware', audio: 'none' } }));
    expect(plan.aacByFfmpeg).toBe(true);
    expect(plan.settings.audio?.encoder).toBe('ffmpeg');
    expect(plan.blocked).toBe(false);
  });

  it('measures where the content ends', () => {
    expect(contentEnd([])).toBe(0);
    expect(contentEnd([item('a', textKind(), 1, 2), item('b', textKind(), 0, 1)])).toBe(seconds(3));
  });
});

describe('export notices', () => {
  const info = (state: ExportInfo['state']): ExportInfo => ({
    id: 'export-1',
    jobId: 'job-1',
    projectId: 'project-1',
    sequenceId: 'sequence-1',
    sequenceName: 'Montage',
    presetName: 'YouTube',
    path: 'C:\\Exports\\Montage.mp4',
    settings: { container: 'wav', video: null, audio: null },
    duration: seconds(10),
    createdAt: 1_794_300_000_000,
    hasThumbnail: false,
    state,
  });

  it('announces a completion or a failure once', () => {
    const rendering = info({ kind: 'rendering', progress: 0.5, etaSeconds: 3 });
    const completed = info({ kind: 'completed', sizeBytes: 42, elapsedSeconds: 6, check: null });
    expect(exportNotice(rendering, completed)).toMatchObject({
      tone: 'success',
      sizeBytes: 42,
      warning: null,
    });
    expect(exportNotice(completed, completed)).toBeNull();
    expect(exportNotice(rendering, info({ kind: 'failed', message: 'disk full' }))).toMatchObject({
      tone: 'error',
      message: 'disk full',
    });
    expect(exportNotice(rendering, info({ kind: 'cancelled' }))).toBeNull();
  });

  it('tags notifications and reads file names on every platform', () => {
    const tag = exportNotificationTag(info({ kind: 'queued' }));
    expect(exportIdOfTag(tag)).toBe('export-1');
    expect(exportIdOfTag('task-1')).toBeNull();
    expect(fileNameOf('C:\\Exports\\Montage.mp4')).toBe('Montage.mp4');
    expect(fileNameOf('/home/me/Montage.mov')).toBe('Montage.mov');
    expect(
      renderIssueCount({ missingAssets: ['a'], missingFonts: ['b', 'c'], failedTemplates: [] }),
    ).toBe(3);
  });
});

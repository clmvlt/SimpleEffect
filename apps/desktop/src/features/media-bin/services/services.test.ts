import type { Asset, CoreEvent, JobSnapshot, ThumbnailSheet, WaveformPeaks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { testAsset } from '../testing/assets';
import { codecLabel, displaySize, framesPerSecond, kilohertz } from './asset-details';
import { assetName, filterAssets, needsDecodabilityCheck, sortAssets } from './asset-list';
import { droppedPaths, summarizeImport } from './import-summary';
import {
  applyJobEvent,
  assetJobStatus,
  IDLE_STATUS,
  jobsFromList,
  latestCompletion,
} from './job-status';
import { tileBackground, tileIndexAt } from './thumbnail-scrub';
import { waveformPath } from './waveform-path';

describe('asset list', () => {
  const clip = testAsset({ id: 'b', originalPath: 'C:\\Rushes\\Été plage.mp4' });
  const song = testAsset({ id: 'a', originalPath: '/music/song.wav', kind: 'audio' });

  it('names assets after their file, whatever the separator', () => {
    expect(assetName(clip)).toBe('Été plage.mp4');
    expect(assetName(song)).toBe('song.wav');
  });

  it('sorts by id (import order) and filters by kind and accent-insensitive name', () => {
    const sorted = sortAssets({ b: clip, a: song });
    expect(sorted.map((asset) => asset.id)).toEqual(['a', 'b']);
    expect(filterAssets(sorted, 'video', '')).toEqual([clip]);
    expect(filterAssets(sorted, 'all', 'ete')).toEqual([clip]);
    expect(filterAssets(sorted, 'audio', 'plage')).toEqual([]);
  });

  it('checks decodability only for present, probed, unchecked assets', () => {
    expect(needsDecodabilityCheck(clip)).toBe(true);
    const checked: Asset = { ...clip, decodability: { video: 'decodable', audio: 'decodable' } };
    expect(needsDecodabilityCheck(checked)).toBe(false);
    expect(needsDecodabilityCheck({ ...clip, missing: true })).toBe(false);
  });
});

describe('asset details', () => {
  const stream = testAsset({}).mediaInfo?.videoStream;
  if (!stream) throw new Error('the test asset has a picture');

  it('names codecs and profiles', () => {
    expect(codecLabel('h264', 'High')).toBe('H.264 High');
    expect(codecLabel('pcm_s24le', null)).toBe('PCM S24LE');
    expect(codecLabel('cfhd', null)).toBe('CFHD');
  });

  it('shows rotated pictures in their display orientation', () => {
    expect(displaySize(stream)).toEqual({ width: 1920, height: 1080 });
    expect(displaySize({ ...stream, rotation: 90 })).toEqual({ width: 1080, height: 1920 });
  });

  it('rounds frame and sample rates', () => {
    expect(framesPerSecond({ ...stream, frameRate: { numerator: 30000, denominator: 1001 } })).toBe(
      '29.97',
    );
    expect(framesPerSecond({ ...stream, frameRate: null })).toBeNull();
    expect(kilohertz({ codec: 'aac', profile: null, channels: 2, sampleRate: 44_100 })).toBe(
      '44.1',
    );
  });
});

describe('job status', () => {
  const job = (id: string, state: JobSnapshot['state'], kind: JobSnapshot['kind'] = 'thumbnails') =>
    ({ id, projectId: 'p', assetId: 'asset', kind, state, attempt: 1 }) satisfies JobSnapshot;

  it('follows job events', () => {
    const queued = job('j1', { kind: 'queued' });
    let jobs = applyJobEvent({}, { kind: 'jobQueued', job: queued });
    const progress: CoreEvent = {
      kind: 'jobProgress',
      jobId: 'j1',
      projectId: 'p',
      assetId: 'asset',
      progress: 0.5,
    };
    jobs = applyJobEvent(jobs, progress);
    expect(assetJobStatus(jobs, 'asset').running).toEqual([{ kind: 'thumbnails', progress: 0.5 }]);
    jobs = applyJobEvent(jobs, { kind: 'jobCompleted', job: job('j1', { kind: 'completed' }) });
    expect(assetJobStatus(jobs, 'asset')).toEqual({ running: [], queued: [], failed: [] });
    expect(latestCompletion(jobs, 'asset', 'thumbnails')).toBe('j1');
    expect(applyJobEvent(jobs, { kind: 'coreShuttingDown' })).toBe(jobs);
  });

  it('keeps only the latest job of each kind', () => {
    const jobs = jobsFromList([
      job('j1', { kind: 'failed', message: 'boom' }),
      job('j2', { kind: 'queued' }),
      job('j3', { kind: 'failed', message: 'no sound' }, 'waveformPeaks'),
    ]);
    expect(assetJobStatus(jobs, 'asset')).toEqual({
      running: [],
      queued: ['thumbnails'],
      failed: [{ kind: 'waveformPeaks', message: 'no sound' }],
    });
    expect(assetJobStatus(jobs, 'other')).toBe(IDLE_STATUS);
  });
});

describe('thumbnail scrub', () => {
  const sheet: ThumbnailSheet = {
    tileWidth: 160,
    tileHeight: 90,
    columns: 8,
    rows: 2,
    times: [],
    posterWidth: 640,
    posterHeight: 360,
  };

  it('maps the pointer to a tile', () => {
    expect(tileIndexAt(0, 12)).toBe(0);
    expect(tileIndexAt(0.5, 12)).toBe(6);
    expect(tileIndexAt(1, 12)).toBe(11);
    expect(tileIndexAt(Number.NaN, 12)).toBe(0);
  });

  it('positions the sheet so one tile fills the box', () => {
    expect(tileBackground(sheet, 9, 80)).toEqual({
      size: '640px 90px',
      position: '-80px -45px',
      height: 45,
    });
  });
});

describe('waveform path', () => {
  const peaks: WaveformPeaks = {
    channels: 1,
    sampleRate: 48_000,
    samplesPerPeak: 1024,
    peakCount: 2,
    values: new Int16Array([-32767, 32767, 0, 0]),
  };

  it('draws one bar per column', () => {
    expect(waveformPath(peaks, 2, 10)).toBe('M0.5 0.0V10.0M1.5 5.0V5.5');
    expect(waveformPath({ ...peaks, peakCount: 0, values: new Int16Array() }, 2, 10)).toBe('');
  });
});

describe('import summary', () => {
  it('counts and names failures', () => {
    const summary = summarizeImport({
      imported: ['a', 'b'],
      duplicates: [{ path: '/x.mp4', assetId: 'c' }],
      failures: [{ path: 'C:\\notes.txt', code: 'MEDIA_UNSUPPORTED', message: '…' }],
    });
    expect(summary).toEqual({
      imported: 2,
      duplicates: 1,
      failures: [{ name: 'notes.txt', code: 'MEDIA_UNSUPPORTED' }],
    });
  });

  it('keeps dropped files that are on disk', () => {
    const files = [new File([], 'a.mp4'), new File([], 'web.png')];
    const paths = droppedPaths(files, (file) => (file.name === 'a.mp4' ? '/a.mp4' : ''));
    expect(paths).toEqual(['/a.mp4']);
  });
});

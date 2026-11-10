/**
 * Exports (prompt 27, ADR-0038) rendered on this page with the export's own renderer, encoder and
 * muxer (VP9 + Opus in WebM: the test Chromium has no H.264 or AAC encoder), then decoded back:
 * every frame is the right one, matches the preview at the same instant within compression
 * noise, and the sound lasts the exported span.
 */
import type { Flicks } from '@se/protocol';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import { MediabunnyVideoSource } from '../../src/decode/video-source';
import { ExportAudioMixer } from '../../src/export/audio-mixer';
import { createExportRenderer } from '../../src/export/export-renderer';
import { renderExport, type ExportRenderJob } from '../../src/export/render-export';
import type { PlayableTrack } from '../../src/audio/audio-clips';
import {
  assetTable,
  clip,
  frameTime,
  item,
  rectangle,
  seconds,
  videoAsset,
} from '../../src/testing/document';
import count25 from '../golden/fixtures/count-25.webm?url';
import {
  comparePixels,
  createTestEngine,
  downsample,
  frameAt,
  readFrameNumber,
} from '../golden/golden';
import { MemorySink, decodeExport, soundAsset, toneWav } from './export-fixtures';

const RATE = { numerator: 25, denominator: 1 };
const FULL_FRAME = { x: 0, y: 0, width: 320, height: 180 };
/** Compression noise allowed against the preview (4 × 4 block averages). */
const EXPORT_TOLERANCE = { channel: 14, ratio: 0.02 };

const pal = videoAsset({ width: 320, height: 180, frameRate: RATE });
const tone = soundAsset(3);
const assets = assetTable(pal, tone);
let toneUrl = '';

beforeAll(() => {
  toneUrl = URL.createObjectURL(toneWav(3, 440));
});
afterAll(() => {
  URL.revokeObjectURL(toneUrl);
});

const files = () => ({ [pal.id]: { original: count25 }, [tone.id]: { original: toneUrl } });
const locate = (assetId: string) => files()[assetId]?.original ?? null;

function track(id: string, kind: 'video' | 'audio', items: PlayableTrack['items']): PlayableTrack {
  return { id, kind, muted: false, solo: false, volumeDb: 0, pan: 0, hidden: false, items };
}

/** V1: the counting clip fading in; V2: a red square at the bottom right; A1: the tone. */
const scene = {
  sequence: {
    format: { width: 320, height: 180 },
    tracks: [
      track('v1', 'video', [item(0 as Flicks, seconds(2), clip(pal.id, { fadeIn: seconds(0.4) }))]),
      track('v2', 'video', [
        item(
          0 as Flicks,
          seconds(2),
          // Below the frame-number bits (read at a quarter of the height).
          rectangle('#ff0000', 240, 240, { transform: { position: { x: 0.8, y: 0.75 } } }),
        ),
      ]),
      track('a1', 'audio', [item(0 as Flicks, seconds(2), clip(tone.id))]),
    ],
  },
  assets,
};

const job: ExportRenderJob = {
  settings: {
    container: 'webm',
    video: { codec: 'vp9', width: 320, height: 180, rate: { kind: 'quality', level: 'veryHigh' } },
    audio: { codec: 'opus', bitsPerSecond: 128_000, encoder: 'webCodecs' },
  },
  pipeline: { kind: 'muxed' },
  start: 0 as Flicks,
  end: seconds(2),
  frameRate: RATE,
};

/** Renders `job` of `scene` into memory with the export's pipeline. */
async function exportToMemory(exportJob: ExportRenderJob): Promise<MemorySink> {
  const sink = new MemorySink();
  const renderer = await createExportRenderer(
    { settings: exportJob.settings, scene, fonts: [] },
    (assetId) => locate(assetId),
    { createVideoSource: (url) => new MediabunnyVideoSource(url, 'prefer-software') },
  );
  const mixer = new ExportAudioMixer({
    sequence: scene.sequence,
    assets,
    locator: (assetId) => locate(assetId),
    start: exportJob.start,
    end: exportJob.end,
  });
  try {
    await renderExport(exportJob, {
      canvas: renderer.canvas,
      draw: (time) => renderer.drawer?.draw(time) ?? Promise.resolve(),
      readRgba: () => renderer.readRgba(),
      sink,
      sound: async (index) => (await mixer.mix(index)).channels,
      thumbnail: async () => {
        const jpeg = await renderer.thumbnail();
        if (jpeg !== null) await sink.thumbnail(jpeg);
      },
      onProgress: () => undefined,
      signal: new AbortController().signal,
    });
  } finally {
    renderer.destroy();
    mixer.destroy();
  }
  return sink;
}

/** The RGBA values of `frame` at (`x`, `y`). */
function pixelAt(frame: ImageData | undefined, x: number, y: number): number[] {
  if (frame === undefined) return [];
  const offset = (y * frame.width + x) * 4;
  return Array.from(frame.data.subarray(offset, offset + 4));
}

describe('video export', () => {
  it('writes every frame, as the preview draws it, with its sound', async () => {
    const sink = await exportToMemory(job);
    const decoded = await decodeExport(sink.streams.data);
    expect(decoded.videoCodec).toBe('vp9');
    expect(decoded.audioCodec).toBe('opus');
    expect(decoded.frames).toHaveLength(50);
    expect(decoded.timestamps[25]).toBeCloseTo(1, 3);
    expect(decoded.audioSeconds).toBeGreaterThan(1.98);
    expect(decoded.audioSeconds).toBeLessThan(2.03);
    expect(sink.thumbnails).toHaveLength(1);

    const { engine, canvas } = await createTestEngine(scene, files());
    try {
      for (const index of [12, 13, 25, 37, 49]) {
        const exported = decoded.frames[index];
        if (exported === undefined) throw new Error(`frame ${String(index)} is missing`);
        expect(readFrameNumber(exported, FULL_FRAME), `frame ${String(index)}`).toBe(index);
        const preview = await frameAt(engine, canvas, frameTime(index, RATE));
        const result = comparePixels(
          downsample(exported, 4),
          downsample(preview, 4),
          EXPORT_TOLERANCE,
        );
        expect(result.ratio, `frame ${String(index)} differs from the preview`).toBeLessThanOrEqual(
          EXPORT_TOLERANCE.ratio,
        );
      }
      // The square of V2 is over the clip, where the preview draws it.
      const [red = 0, green = 255] = pixelAt(decoded.frames[25], 256, 135);
      expect(red).toBeGreaterThan(200);
      expect(green).toBeLessThan(60);
      // During the fade from black, the export is as dark as the preview.
      const early = decoded.frames[2];
      const previewEarly = await frameAt(engine, canvas, frameTime(2, RATE));
      if (early === undefined) throw new Error('frame 2 is missing');
      expect(
        comparePixels(downsample(early, 4), downsample(previewEarly, 4), EXPORT_TOLERANCE).ratio,
      ).toBeLessThanOrEqual(EXPORT_TOLERANCE.ratio);
    } finally {
      engine.destroy();
    }
  });

  it('exports an in/out span from its first frame', async () => {
    const sink = await exportToMemory({ ...job, start: seconds(1), end: seconds(1.4) });
    const decoded = await decodeExport(sink.streams.data);
    expect(decoded.frames).toHaveLength(10);
    expect(decoded.timestamps[0]).toBeCloseTo(0, 6);
    expect(readFrameNumber(decoded.frames[0] ?? new ImageData(1, 1), FULL_FRAME)).toBe(25);
    expect(readFrameNumber(decoded.frames[9] ?? new ImageData(1, 1), FULL_FRAME)).toBe(34);
  });

  it('sends masters to ffmpeg as raw RGBA frames after their WAV', async () => {
    const sink = await exportToMemory({
      ...job,
      settings: {
        container: 'mov',
        video: {
          ...job.settings.video,
          codec: 'prores422Hq',
        } as ExportRenderJob['settings']['video'],
        audio: { codec: 'pcm', bitsPerSecond: 0, encoder: 'webCodecs' },
      },
      pipeline: { kind: 'ffmpegFrames', withAudio: true },
      end: seconds(0.6),
    });
    expect(sink.frames).toHaveLength(15);
    expect(sink.frames[0]).toHaveLength(320 * 180 * 4);
    const frame = new ImageData(new Uint8ClampedArray(sink.frames[14] ?? []), 320, 180);
    expect(readFrameNumber(frame, FULL_FRAME)).toBe(14);
    expect(String.fromCharCode(...sink.streams.audio.subarray(8, 12))).toBe('WAVE');
  });
});

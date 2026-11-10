import type { Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { NTSC_30, seconds } from '../testing/document';
import { FLICKS_PER_SECOND } from '../time/flicks';
import { maximumAudioPackets, maximumVideoPackets, webCodecsVideoCodec } from './export-codecs';
import { audioBlocks, exportFrames, sampleCount } from './export-timing';
import { withoutUrls } from './export-job-runner';

describe('export frames', () => {
  it('counts the frames starting before the end, at whole flicks', () => {
    const frames = exportFrames(seconds(1), seconds(3), { numerator: 25, denominator: 1 });
    expect(frames.count).toBe(50);
    expect(frames.timeOf(0)).toBe(seconds(1));
    expect(frames.timeOf(25)).toBe(seconds(2));
    expect(frames.secondsOf(25)).toBe(1);
    expect(frames.frameSeconds).toBe(0.04);
  });

  it('keeps NTSC frames exact over a long span', () => {
    const frames = exportFrames(0 as Flicks, seconds(600), NTSC_30);
    expect(frames.count).toBe(17_983);
    expect(frames.timeOf(17_982)).toBeLessThan(seconds(600));
    expect(frames.secondsOf(30_000)).toBe(1001);
    expect(Number.isInteger(frames.timeOf(12_345))).toBe(true);
  });

  it('gives no frame to an empty span', () => {
    expect(exportFrames(seconds(2), seconds(2), NTSC_30).count).toBe(0);
  });
});

describe('export sound blocks', () => {
  it('cuts the span into whole-sample blocks adding up to the exact duration', () => {
    const start = 12_345 as Flicks;
    const end = (start + 65.5 * FLICKS_PER_SECOND) as Flicks;
    const blocks = audioBlocks(start, end, 48_000, 30);
    expect(blocks.map((block) => block.frames)).toEqual([1_440_000, 1_440_000, 264_000]);
    expect(blocks.map((block) => block.seconds)).toEqual([0, 30, 60]);
    expect(blocks[1]?.start).toBe(start + 30 * FLICKS_PER_SECOND);
    expect(blocks.reduce((sum, block) => sum + block.frames, 0)).toBe(
      sampleCount(start, end, 48_000),
    );
  });

  it('rounds a span ending between samples to the nearest sample', () => {
    expect(sampleCount(0 as Flicks, (14_700 * 10 + 7_000) as Flicks, 48_000)).toBe(10);
    expect(sampleCount(0 as Flicks, (14_700 * 10 + 7_400) as Flicks, 48_000)).toBe(11);
  });
});

describe('export codecs', () => {
  it('encodes delivery codecs with WebCodecs and leaves masters to ffmpeg', () => {
    expect(webCodecsVideoCodec('h264')).toBe('avc');
    expect(webCodecsVideoCodec('vp9')).toBe('vp9');
    expect(webCodecsVideoCodec('prores422Hq')).toBeNull();
    expect(webCodecsVideoCodec('dnxhrHq')).toBeNull();
  });

  it('reserves more packets than a file can hold', () => {
    expect(maximumVideoPackets(100)).toBeGreaterThan(100);
    // AAC: one packet per 1024 samples; Opus: one per 20 ms.
    expect(maximumAudioPackets(60)).toBeGreaterThan((60 * 48_000) / 1024);
    expect(maximumAudioPackets(60)).toBeGreaterThan(60 / 0.02);
  });
});

describe('error messages', () => {
  it('never carry URLs (they hold the session token)', () => {
    expect(
      withoutUrls('fetch http://127.0.0.1:1234/media/exports/x/assets/a/original?t=secret failed'),
    ).toBe('fetch <url> failed');
  });
});

/**
 * The export's block mixing (prompt 27, ADR-0038) with real Web Audio and Mediabunny decoding: a
 * mix in short blocks equals the same mix in one pass (no click at block boundaries), lasts
 * exactly the exported span, and applies the preview's levels (volume, fades, mute).
 */
import type { Flicks } from '@se/protocol';
import { afterAll, beforeAll, describe, expect, it } from 'vitest';

import type { PlayableSequence, PlayableTrack } from '../../src/audio/audio-clips';
import { ExportAudioMixer } from '../../src/export/audio-mixer';
import { sampleCount } from '../../src/export/export-timing';
import {
  NTSC_30,
  assetTable,
  clip,
  frameTime,
  item,
  seconds,
  still,
} from '../../src/testing/document';
import { soundAsset, toneWav } from './export-fixtures';

const tone = soundAsset(3);
const assets = assetTable(tone);
let url = '';

beforeAll(() => {
  url = URL.createObjectURL(toneWav(3, 440));
});
afterAll(() => {
  URL.revokeObjectURL(url);
});

function track(id: string, items: PlayableTrack['items'], muted = false): PlayableTrack {
  return { id, kind: 'audio', muted, solo: false, volumeDb: 0, pan: 0, hidden: false, items };
}

/**
 * A1: the tone from 0 with a linear fade in and −6 dB, then again from a 29.97 fps frame
 * boundary (a start between two samples); A2 (muted): the tone where A1 is silent.
 */
const scene: PlayableSequence = {
  tracks: [
    track('a1', [
      item(0 as Flicks, seconds(1.2), clip(tone.id, { fadeIn: seconds(0.3), volumeDb: still(-6) })),
      item(frameTime(37, NTSC_30), seconds(0.9), clip(tone.id, { sourceStart: seconds(0.5) })),
    ]),
    track('a2', [item(seconds(2.3), seconds(0.5), clip(tone.id))], true),
  ],
};

function mixer(blockSeconds: number, start: Flicks, end: Flicks): ExportAudioMixer {
  return new ExportAudioMixer({
    sequence: scene,
    assets,
    locator: () => url,
    start,
    end,
    blockSeconds,
  });
}

/** Every block of `mixer`, joined per channel. */
async function mixAll(source: ExportAudioMixer): Promise<Float32Array[]> {
  const total = source.blocks.reduce((sum, block) => sum + block.frames, 0);
  const joined = [new Float32Array(total), new Float32Array(total)];
  let at = 0;
  for (const block of source.blocks) {
    const { channels } = await source.mix(block.index);
    joined.forEach((channel, index) => {
      channel.set(channels[index] ?? new Float32Array(0), at);
    });
    at += block.frames;
  }
  source.destroy();
  return joined;
}

function peak(samples: Float32Array, from: number, to: number): number {
  let largest = 0;
  for (let index = from; index < to; index++) {
    largest = Math.max(largest, Math.abs(samples[index] ?? 0));
  }
  return largest;
}

describe('export block mixing', () => {
  it('equals a one-pass mix, without clicks at block boundaries', async () => {
    const start = 0 as Flicks;
    const end = seconds(2.8);
    const [blocked, whole] = await Promise.all([
      mixAll(mixer(0.4, start, end)),
      mixAll(mixer(10, start, end)),
    ]);
    for (const [channel, samples] of blocked.entries()) {
      const reference = whole[channel] ?? new Float32Array(0);
      expect(samples).toHaveLength(reference.length);
      let largest = 0;
      for (let index = 0; index < samples.length; index++) {
        largest = Math.max(largest, Math.abs((samples[index] ?? 0) - (reference[index] ?? 0)));
      }
      expect(largest, `channel ${String(channel)}`).toBeLessThan(1e-4);
    }
  });

  it('lasts exactly the exported span, even starting between samples', async () => {
    const start = 7_777 as Flicks;
    const end = (start + seconds(2.05)) as Flicks;
    const [left] = await mixAll(mixer(0.5, start, end));
    expect(left).toHaveLength(sampleCount(start, end, 48_000));
  });

  it('applies the preview levels: fade in, volume and mute', async () => {
    const [left = new Float32Array(0)] = await mixAll(mixer(0.4, 0 as Flicks, seconds(2.8)));
    const at = (time: number) => Math.round(time * 48_000);
    // Rising through the fade, then −6 dB of a half-scale tone (~0.25 per speaker side).
    expect(peak(left, at(0), at(0.05))).toBeLessThan(peak(left, at(0.25), at(0.3)));
    expect(peak(left, at(0.5), at(1.1))).toBeGreaterThan(0.15);
    expect(peak(left, at(0.5), at(1.1))).toBeLessThan(0.4);
    // A2 is muted: silence where only it plays.
    expect(peak(left, at(2.35), at(2.75))).toBeLessThan(1e-4);
  });
});

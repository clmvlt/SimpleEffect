import type { Flicks, FrameRate } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import flicksJson from '../../../../../docs/test-vectors/flicks.json?raw';
import timecodeJson from '../../../../../docs/test-vectors/timecode.json?raw';
import { flicksToFrames, framesToFlicks, MAX_SAFE_FLICKS, secondsToFlicks } from './flicks';
import { frameDuration, frameRate } from './frame-rate';
import { formatTimecode, parseTimecode } from './timecode';

/**
 * Parity with Rust: the shared vectors also run in `crates/se-domain/tests/test_vectors.rs`,
 * so both sides convert, parse and format identically (ADR-0004).
 */

interface FlicksVectors {
  readonly flicksPerSecond: number;
  readonly maxSafe: number;
  readonly frameDurations: readonly { frameRate: FrameRate; flicks: number }[];
  readonly reducedFrameRates: readonly { input: FrameRate; reduced: FrameRate }[];
  readonly invalidFrameRates: readonly { frameRate: FrameRate; error: string }[];
  readonly framesToFlicks: readonly { frameRate: FrameRate; frames: number; flicks: number }[];
  readonly flicksToFrames: readonly {
    frameRate: FrameRate;
    flicks: number;
    frames: { floor: number; ceil: number; nearest: number };
  }[];
  readonly secondsToFlicks: readonly { seconds: number; flicks: number }[];
  readonly secondsToFlicksErrors: readonly { seconds: number; error: string }[];
}

interface TimecodeVectors {
  readonly parse: readonly { input: string; frameRate: FrameRate | null; flicks: number }[];
  readonly parseErrors: readonly { input: string; frameRate: FrameRate | null; error: string }[];
  readonly format: readonly { flicks: number; frameRate: FrameRate; timecode: string }[];
}

const flicks = JSON.parse(flicksJson) as FlicksVectors;
const timecode = JSON.parse(timecodeJson) as TimecodeVectors;

describe('flicks vectors', () => {
  it('agree on the constants', () => {
    expect(flicks.flicksPerSecond).toBe(705_600_000);
    expect(flicks.maxSafe).toBe(MAX_SAFE_FLICKS);
  });

  it.each(flicks.frameDurations)(
    'frame duration at $frameRate.numerator/$frameRate.denominator',
    (vector) => {
      expect(frameDuration(vector.frameRate)).toBe(vector.flicks);
    },
  );

  it.each(flicks.reducedFrameRates)('reduces $input.numerator/$input.denominator', (vector) => {
    expect(frameRate(vector.input.numerator, vector.input.denominator)).toEqual({
      ok: true,
      value: vector.reduced,
    });
  });

  it.each(flicks.invalidFrameRates)(
    'refuses $frameRate.numerator/$frameRate.denominator',
    (vector) => {
      expect(frameRate(vector.frameRate.numerator, vector.frameRate.denominator)).toEqual({
        ok: false,
        error: vector.error,
      });
    },
  );

  it.each(flicks.framesToFlicks)('frame $frames → $flicks', (vector) => {
    expect(framesToFlicks(vector.frames, vector.frameRate)).toEqual({
      ok: true,
      value: vector.flicks,
    });
  });

  it.each(flicks.flicksToFrames)('$flicks flicks → frames', (vector) => {
    const time = vector.flicks as Flicks;
    expect(flicksToFrames(time, vector.frameRate, 'floor')).toBe(vector.frames.floor);
    expect(flicksToFrames(time, vector.frameRate, 'ceil')).toBe(vector.frames.ceil);
    expect(flicksToFrames(time, vector.frameRate, 'nearest')).toBe(vector.frames.nearest);
  });

  it.each(flicks.secondsToFlicks)('$seconds s → $flicks', (vector) => {
    expect(secondsToFlicks(vector.seconds)).toEqual({ ok: true, value: vector.flicks });
  });

  it.each(flicks.secondsToFlicksErrors)('$seconds s is refused', (vector) => {
    expect(secondsToFlicks(vector.seconds)).toEqual({ ok: false, error: vector.error });
  });
});

describe('timecode vectors', () => {
  it.each(timecode.parse)('parses "$input"', (vector) => {
    expect(parseTimecode(vector.input, vector.frameRate)).toEqual({
      ok: true,
      value: vector.flicks,
    });
  });

  it.each(timecode.parseErrors)('refuses "$input"', (vector) => {
    expect(parseTimecode(vector.input, vector.frameRate)).toEqual({
      ok: false,
      error: vector.error,
    });
  });

  it.each(timecode.format)('formats $flicks as $timecode', (vector) => {
    expect(formatTimecode(vector.flicks as Flicks, vector.frameRate)).toBe(vector.timecode);
  });
});

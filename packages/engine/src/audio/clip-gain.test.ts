import type { Animatable, AssetId, Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { clip, crossFade, item, seconds, still } from '../testing/document';
import type { EvaluableItem } from '../evaluate/evaluate';
import type { EvaluableTransition } from '../evaluate/transitions';
import { audioClipsOf, type AudioClip } from './audio-clips';
import { scheduleVoiceGain } from './audio-graph';
import type { AudioVoiceRequest } from './audio-scheduler';
import { GAIN_STEP_SECONDS, clipGainAt, gainEnvelope } from './clip-gain';
import { dbToGain } from './mix';

const SOUND = 'asset-sound' as AssetId;

/** The clips of one audio track holding `items` and `transitions`. */
function clipsOf(
  items: readonly EvaluableItem[],
  transitions: readonly EvaluableTransition[] = [],
): AudioClip[] {
  return audioClipsOf({
    tracks: [
      {
        id: 'a1',
        kind: 'audio',
        muted: false,
        solo: false,
        volumeDb: 0,
        pan: 0,
        hidden: false,
        items,
        transitions,
      },
    ],
  });
}

function onlyClip(options: Parameters<typeof clip>[1], length = seconds(10)): AudioClip {
  const [single] = clipsOf([item(0 as Flicks, length, clip(SOUND, options))]);
  if (single === undefined) throw new Error('no clip');
  return single;
}

/** A volume going from `from` dB at 0 s to `to` dB at 2 s, linearly in decibels. */
function ramp(from: number, to: number): Animatable<number> {
  return {
    kind: 'keyframed',
    keyframes: [
      { time: 0 as Flicks, value: from, easing: { kind: 'linear' } },
      { time: seconds(2), value: to, easing: { kind: 'linear' } },
    ],
  };
}

describe('clip gain', () => {
  it('converts decibels to linear gain', () => {
    expect(dbToGain(0)).toBe(1);
    expect(dbToGain(-6)).toBeCloseTo(0.501, 3);
    expect(clipGainAt(onlyClip({ volumeDb: still(-12) }), seconds(3))).toBeCloseTo(0.251, 3);
  });

  it('interpolates volume keyframes in decibels, then converts', () => {
    const fading = onlyClip({ volumeDb: ramp(-6, -18) });
    expect(clipGainAt(fading, seconds(1))).toBeCloseTo(dbToGain(-12), 9);
    // Held before the first and after the last keyframe.
    expect(clipGainAt(fading, seconds(5))).toBeCloseTo(dbToGain(-18), 9);
  });

  it('multiplies the volume by the fades along their shapes', () => {
    const faded = onlyClip({
      volumeDb: still(-6),
      fadeIn: seconds(1),
      fadeInShape: 'constantPower',
      fadeOut: seconds(2),
    });
    expect(clipGainAt(faded, seconds(0.5))).toBeCloseTo(dbToGain(-6) * Math.SQRT1_2, 9);
    expect(clipGainAt(faded, seconds(9))).toBeCloseTo(dbToGain(-6) * 0.5, 9);
    expect(clipGainAt(faded, 0)).toBe(0);
  });
});

describe('gain envelopes', () => {
  it('are one point while the gain holds', () => {
    expect(gainEnvelope(onlyClip({ volumeDb: still(-6) }), seconds(1), seconds(2))).toEqual([
      { at: 0, gain: dbToGain(-6) },
    ]);
  });

  it('sample a varying gain every step, ending exactly at the end', () => {
    const envelope = gainEnvelope(onlyClip({ volumeDb: ramp(0, -24) }), 0, seconds(1.005));
    expect(envelope).toHaveLength(Math.ceil(1.005 / GAIN_STEP_SECONDS) + 1);
    expect(envelope[1]?.at).toBeCloseTo(GAIN_STEP_SECONDS, 12);
    const last = envelope.at(-1);
    expect(last?.at).toBeCloseTo(1.005, 12);
    expect(last?.gain).toBeCloseTo(dbToGain(-12.06), 9);
    // Linear ramps between points stay within 0.1 dB of the curve.
    const [first, second] = envelope;
    if (first === undefined || second === undefined) throw new Error('points expected');
    const middle = (first.gain + second.gain) / 2;
    const exact = clipGainAt(onlyClip({ volumeDb: ramp(0, -24) }), seconds(0.005));
    expect(Math.abs(20 * Math.log10(middle / exact))).toBeLessThan(0.1);
  });

  it('become Web Audio ramps', () => {
    const calls: string[] = [];
    const param = {
      value: 1,
      setValueAtTime: (value: number, time: number) => {
        calls.push(`set ${value.toFixed(2)} @${time.toFixed(2)}`);
        return param as unknown as AudioParam;
      },
      linearRampToValueAtTime: (value: number, time: number) => {
        calls.push(`ramp ${value.toFixed(2)} @${time.toFixed(2)}`);
        return param as unknown as AudioParam;
      },
    };
    const request = {
      when: 10,
      duration: 1,
      playbackRate: 1,
      fade: 0,
      gain: [
        { at: 0, gain: 1 },
        { at: 0.5, gain: 0.5 },
        { at: 1, gain: 0.25 },
      ],
    } as unknown as AudioVoiceRequest;
    scheduleVoiceGain(param, request);
    expect(calls).toEqual(['set 1.00 @10.00', 'ramp 0.50 @10.50', 'ramp 0.25 @11.00']);
    calls.length = 0;
    scheduleVoiceGain(param, { ...request, gain: [{ at: 0, gain: 0.3 }] });
    expect(calls).toEqual([]);
    expect(param.value).toBe(0.3);
  });
});

describe('sound cross-fades', () => {
  const outgoing = item(0 as Flicks, seconds(4), clip(SOUND, { sourceStart: seconds(10) }));
  const incoming = item(seconds(4), seconds(4), clip(SOUND, { sourceStart: seconds(20) }));
  const fade = crossFade(outgoing, incoming, seconds(2), { kind: 'audioCrossfade' });

  it('extend both clips through the span', () => {
    const [out, into] = clipsOf([outgoing, incoming], [fade]);
    expect(out?.playEnd).toBe(seconds(5));
    expect(into?.playStart).toBe(seconds(3));
    expect(out?.crossfadeOut).toEqual({ start: seconds(3), end: seconds(5) });
    expect(into?.crossfadeIn).toEqual({ start: seconds(3), end: seconds(5) });
  });

  it('weigh the sides at constant power', () => {
    const [out, into] = clipsOf([outgoing, incoming], [fade]);
    if (out === undefined || into === undefined) throw new Error('clips expected');
    for (const time of [seconds(3.2), seconds(4), seconds(4.7)]) {
      const sum = clipGainAt(out, time) ** 2 + clipGainAt(into, time) ** 2;
      expect(sum).toBeCloseTo(1, 9);
    }
    expect(clipGainAt(out, seconds(5))).toBe(0);
    expect(clipGainAt(into, seconds(2.9))).toBe(0);
  });

  it('are ignored when they are picture dissolves', () => {
    const dissolve = crossFade(outgoing, incoming, seconds(2));
    const [out] = clipsOf([outgoing, incoming], [dissolve]);
    expect(out?.playEnd).toBe(seconds(4));
  });
});

import type { Flicks } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { seconds } from '../testing/document';
import { frameStart } from './frame-pacing';
import { PlaybackController, type FrameTicker } from './playback-controller';
import type { PlaybackBounds, Transport } from './transport';

const RATE = { numerator: 25, denominator: 1 };

/** A clock and a display refresh driven by the test. */
function harness() {
  let now = 0;
  let pending: (() => void) | null = null;
  const frames: Flicks[] = [];
  const transports: { transport: Transport | null; bounds: PlaybackBounds }[] = [];
  const ticker: FrameTicker = {
    request: (callback) => {
      pending = callback;
      return 1;
    },
    cancel: () => {
      pending = null;
    },
  };
  const controller = new PlaybackController({
    clock: { now: () => now },
    frameRate: RATE,
    ticker,
    onFrame: (time) => frames.push(time),
    onTransportChange: (transport, bounds) => transports.push({ transport, bounds }),
  });
  controller.setContentEnd(seconds(10));
  return {
    controller,
    frames,
    transports,
    /** Advances the clock by `delta` seconds, then runs the display refresh. */
    refresh: (delta: number) => {
      now += delta;
      const callback = pending;
      pending = null;
      callback?.();
    },
    get ticking() {
      return pending !== null;
    },
  };
}

describe('PlaybackController', () => {
  it('shows the frame the clock says and skips late frames', () => {
    const { controller, frames, refresh } = harness();
    controller.play();
    refresh(0.05);
    refresh(0.04);
    // The display stalled for 200 ms: frames 3 to 6 are skipped.
    refresh(0.2);
    expect(frames).toEqual([frameStart(1, RATE), frameStart(2, RATE), frameStart(7, RATE)]);
    expect(controller.getState().rate).toBe(1);
  });

  it('pauses on the frame on screen and stops back at the start', () => {
    const { controller, frames, refresh, ticking } = harness();
    controller.seek(seconds(2));
    controller.play();
    refresh(0.49);
    controller.pause();
    expect(controller.currentTime()).toBe(seconds(2.48));
    expect(ticking).toBe(false);
    controller.play();
    refresh(0.1);
    controller.stop();
    expect(controller.currentTime()).toBe(seconds(2.48));
    expect(frames.at(-1)).toBe(seconds(2.48));
  });

  it('runs JKL: speeds, reverse showing the current frame first, frame steps', () => {
    const { controller, refresh } = harness();
    controller.seek(seconds(4));
    controller.shuttle('forward');
    controller.shuttle('forward');
    expect(controller.getState().rate).toBe(2);
    refresh(0.51);
    expect(controller.currentTime()).toBe(seconds(5));
    controller.shuttle('reverse');
    expect(controller.getState().rate).toBe(-1);
    refresh(0.039);
    expect(controller.currentTime()).toBe(seconds(4.96));
    controller.step(1);
    expect(controller.getState().rate).toBe(0);
    expect(controller.currentTime()).toBe(seconds(5));
    controller.step(-3);
    expect(controller.currentTime()).toBe(seconds(4.88));
  });

  it('stops at the end, and starts again from zero when played there', () => {
    const { controller, refresh } = harness();
    controller.seek(seconds(9.9));
    controller.play();
    refresh(0.5);
    expect(controller.currentTime()).toBe(seconds(10));
    expect(controller.getState().rate).toBe(0);
    controller.play();
    expect(controller.currentTime()).toBe(0);
  });

  it('loops over the in/out span, entering it when started outside', () => {
    const { controller, refresh, transports } = harness();
    controller.setLoopRange({ start: seconds(2), end: seconds(3) });
    controller.setLooping(true);
    controller.seek(seconds(6));
    controller.play();
    expect(controller.currentTime()).toBe(seconds(2));
    refresh(1.21);
    expect(controller.currentTime()).toBe(seconds(2.2));
    expect(transports.at(-1)?.bounds).toEqual({
      span: { start: seconds(2), end: seconds(3) },
      loop: true,
    });
  });

  it('keeps playing from a seek and re-plans the sound', () => {
    const { controller, refresh, transports } = harness();
    controller.play();
    refresh(0.2);
    const before = transports.length;
    controller.seek(seconds(7));
    expect(transports).toHaveLength(before + 1);
    expect(transports.at(-1)?.transport?.anchorTime).toBe(seconds(7));
    refresh(0.41);
    expect(controller.currentTime()).toBe(seconds(7.4));
  });

  it('refuses reverse playback from the start and playback of an empty sequence', () => {
    const { controller } = harness();
    controller.play(-1);
    expect(controller.getState().rate).toBe(0);
    controller.setContentEnd(0 as Flicks);
    controller.play();
    expect(controller.getState().rate).toBe(0);
  });
});

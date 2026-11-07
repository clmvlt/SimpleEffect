import { describe, expect, it } from 'vitest';

import type { Picture } from '../composite/compositor';
import { FakeFrame } from '../testing/fake-decoding';
import { FrameCache } from './frame-cache';

function picture(index: number): Picture & { readonly frame: FakeFrame } {
  const frame = new FakeFrame(index);
  return { source: frame, rotation: 0, mirrored: false, frame };
}

function cache(maxEntries = 3, budgetBytes = 1_000) {
  return new FrameCache({ budgetBytes, maxEntries }, (entry) => {
    entry.source.close();
  });
}

describe('FrameCache', () => {
  it('finds the picture whose range contains a time, per source', () => {
    const frames = cache();
    frames.insert('a', picture(0), 0, 0.04, 10).release();
    frames.insert('a', picture(1), 0.04, 0.08, 10).release();
    expect(frames.lease('a', 0.05)?.picture.source).toBeInstanceOf(FakeFrame);
    expect(frames.lease('a', 0.08)).toBeNull();
    expect(frames.lease('b', 0.05)).toBeNull();
  });

  it('closes the least recently used pictures beyond the entry limit', () => {
    const frames = cache(2);
    const first = picture(0);
    const second = picture(1);
    frames.insert('a', first, 0, 1, 10).release();
    frames.insert('a', second, 1, 2, 10).release();
    frames.lease('a', 0.5)?.release();
    frames.insert('a', picture(2), 2, 3, 10).release();
    expect(first.frame.closed).toBe(false);
    expect(second.frame.closed).toBe(true);
    expect(frames.stats()).toEqual({ entries: 2, bytes: 20 });
  });

  it('respects the byte budget', () => {
    const frames = cache(10, 25);
    const first = picture(0);
    frames.insert('a', first, 0, 1, 10).release();
    frames.insert('a', picture(1), 1, 2, 10).release();
    frames.insert('a', picture(2), 2, 3, 10).release();
    expect(first.frame.closed).toBe(true);
    expect(frames.stats().bytes).toBe(20);
  });

  it('never closes a leased picture, and closes it on release once removed', () => {
    const frames = cache(1);
    const held = picture(0);
    const lease = frames.insert('a', held, 0, 1, 10);
    frames.insert('a', picture(1), 1, 2, 10).release();
    expect(held.frame.closed).toBe(false);
    frames.dropSource('a');
    expect(held.frame.closed).toBe(false);
    lease.release();
    lease.release();
    expect(held.frame.closed).toBe(true);
  });

  it('keeps the first picture of a source and start, closing a duplicate', () => {
    const frames = cache();
    const original = picture(0);
    const duplicate = picture(0);
    frames.insert('a', original, 0, 1, 10).release();
    const lease = frames.insert('a', duplicate, 0, 1, 10);
    expect(duplicate.frame.closed).toBe(true);
    expect(lease.picture).toBe(original);
    lease.release();
  });

  it('closes everything on clear', () => {
    const frames = cache();
    const pictures = [picture(0), picture(1)];
    pictures.forEach((entry, index) => {
      frames.insert('a', entry, index, index + 1, 10).release();
    });
    frames.clear();
    expect(pictures.every((entry) => entry.frame.closed)).toBe(true);
    expect(frames.stats()).toEqual({ entries: 0, bytes: 0 });
  });
});

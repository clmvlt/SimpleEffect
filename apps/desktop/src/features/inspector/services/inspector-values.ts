import type { Color, FrameRate, Flicks, ItemEntity } from '@se/protocol';

import { frameDuration } from '../../../shared/time';

/** The playhead relative to `item`, clamped into it (keyframe times are item-relative). */
export function localTime(item: Pick<ItemEntity, 'timelineRange'>, playhead: Flicks): Flicks {
  const { start, duration } = item.timelineRange;
  return Math.min(duration, Math.max(0, playhead - start)) as Flicks;
}

/** How close (half a frame) the playhead must be to a keyframe to stand on it. */
export function keyframeTolerance(rate: FrameRate): number {
  return frameDuration(rate) / 2;
}

/** The `#rrggbb` part of a colour (`ColorField` edits opaque colours). */
export function opaquePart(color: Color): string {
  return color.slice(0, 7).toLowerCase();
}

/** `picked` (`#rrggbb`) with the alpha of `previous` kept (`#rrggbbaa` when not opaque). */
export function keepAlpha(picked: string, previous: Color): Color {
  const alpha = previous.length === 9 ? previous.slice(7) : '';
  return `${picked.slice(0, 7).toLowerCase()}${alpha === 'ff' ? '' : alpha}`;
}

/** Whether every value is the same (`equal` compares two). */
export function allSame<T>(values: readonly T[], equal: (left: T, right: T) => boolean): boolean {
  const [first] = values;
  return first === undefined || values.every((value) => equal(first, value));
}

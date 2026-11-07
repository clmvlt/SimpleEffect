import type { Asset, AssetId, Flicks, ItemId } from '@se/protocol';

import type { PrefetchRequest } from '../decode/decoder-pool';
import type { ReadAheadBudget } from '../decode/frame-reader';
import { frameLookupTime } from '../decode/frame-time';
import { chooseVariant, type MediaLocator } from '../decode/media-variant';
import type { EvaluableItem, EvaluableSequence } from '../evaluate/evaluate';
import type { AssetTable } from '../evaluate/item-layer';
import { clipSourceTime } from '../evaluate/timing';
import { playExtensions, trackTransitionSpans } from '../evaluate/transitions';
import { FLICKS_PER_SECOND } from '../time/flicks';

/** Source time of a clip playback will show soon: `[from, to]` in the asset, in flicks. */
export interface PrefetchSpan {
  readonly itemId: ItemId;
  readonly assetId: AssetId;
  readonly from: Flicks;
  readonly to: Flicks;
}

/** The first item of a sorted, non-overlapping list ending after `time` (binary search). */
function firstEndingAfter(items: readonly EvaluableItem[], time: number): number {
  let low = 0;
  let high = items.length;
  while (low < high) {
    const middle = (low + high) >> 1;
    const range = items[middle]?.timelineRange;
    if (range !== undefined && range.start + range.duration <= time) low = middle + 1;
    else high = middle;
  }
  return low;
}

/**
 * The video clips on visible tracks that playback at `rate` from `time` shows during the next
 * `windowSeconds` of real time, with the source time each will need (pure). Forward, the window
 * lies after `time` (upcoming clips included, so a cut does not stall); in reverse, before it.
 * Clips joined by a cross dissolve play beyond their edges: those parts are included.
 */
export function prefetchSpans(
  sequence: EvaluableSequence,
  time: Flicks,
  rate: number,
  windowSeconds: number,
): PrefetchSpan[] {
  if (rate === 0) return [];
  const reach = windowSeconds * Math.abs(rate) * FLICKS_PER_SECOND;
  const low = rate > 0 ? time : time - reach;
  const high = rate > 0 ? time + reach : time + 1;
  const spans: PrefetchSpan[] = [];
  for (const track of sequence.tracks) {
    if (track.kind !== 'video' || track.hidden) continue;
    const extensions = playExtensions(trackTransitionSpans(track.items, track.transitions ?? []));
    // The item before the first one ending after `low` may play past its end (a dissolve).
    const firstIndex = Math.max(0, firstEndingAfter(track.items, low) - 1);
    for (let index = firstIndex; index < track.items.length; index++) {
      const item = track.items[index];
      const extension = item === undefined ? undefined : extensions.get(item.id);
      const before = extension?.before ?? 0;
      if (item === undefined || item.timelineRange.start - before >= high) break;
      if (item.kind.kind !== 'mediaClip') continue;
      const range = item.timelineRange;
      const first = Math.max(low, range.start - before);
      const last = Math.min(high, range.start + range.duration + (extension?.after ?? 0)) - 1;
      if (last < first) continue;
      spans.push({
        itemId: item.id,
        assetId: item.kind.assetId,
        from: clipSourceTime(item.kind, range, Math.round(first) as Flicks),
        to: clipSourceTime(item.kind, range, Math.round(last) as Flicks),
      });
    }
  }
  return spans;
}

/** What turns prefetch spans into decoder-pool requests. */
export interface PrefetchContext {
  readonly assets: AssetTable;
  readonly proxy: boolean;
  readonly locator: MediaLocator;
  /** 1 forward, −1 reverse. */
  readonly direction: 1 | -1;
  /**
   * What one reader may hold: a reverse window is shortened to it, since its reader must decode
   * the whole window before reaching the frame on screen.
   */
  readonly budget: ReadAheadBudget;
}

/** Frames of `asset` a budget holds (by count, and by bytes at the asset's coded size). */
function framesWithin(budget: ReadAheadBudget, asset: Asset): number {
  const stream = asset.mediaInfo?.videoStream;
  const bytes = stream === null || stream === undefined ? 0 : stream.width * stream.height * 4;
  return Math.max(
    1,
    Math.min(budget.frames, bytes > 0 ? Math.floor(budget.bytes / bytes) : budget.frames),
  );
}

/**
 * Decoder-pool requests for `spans`: the file each clip plays (same choice as drawing), in media
 * seconds aimed inside frames, extended by half a frame so the last frame is included. Clips
 * without a playable file are left out (drawing reports them).
 */
export function prefetchRequests(
  spans: readonly PrefetchSpan[],
  context: PrefetchContext,
): PrefetchRequest[] {
  const requests: PrefetchRequest[] = [];
  for (const span of spans) {
    const asset = context.assets[span.assetId];
    if (asset === undefined) continue;
    const choice = chooseVariant(asset, { proxy: context.proxy });
    if (choice.kind !== 'ready') continue;
    const url = context.locator(span.assetId, choice.variant);
    if (url === null) continue;
    const frameRate = asset.mediaInfo?.videoStream?.frameRate ?? null;
    const frame = frameRate === null ? 0.04 : frameRate.denominator / frameRate.numerator;
    const to = frameLookupTime(span.to, frameRate) + frame / 2;
    const from = frameLookupTime(span.from, frameRate);
    requests.push({
      key: span.itemId,
      file: { assetId: span.assetId, variant: choice.variant, url },
      from:
        context.direction < 0
          ? Math.max(from, to - framesWithin(context.budget, asset) * frame)
          : from,
      to,
      direction: context.direction,
    });
  }
  return requests;
}

import type { Flicks, Item, ItemEntity, PastedItem, TrackEntity, TrackKind } from '@se/protocol';

import { trackNumbers, tracksByKind } from './track-numbers';

/** A copied item, with the kind and number of the track it was on. */
export interface ClipboardEntry {
  readonly kind: TrackKind;
  readonly number: number;
  readonly item: Item;
}

/** Items copied in the timeline (internal clipboard, kept for the project they come from). */
export interface Clipboard {
  readonly projectId: string;
  readonly entries: readonly ClipboardEntry[];
}

/** The clipboard holding copies of `items` (on `tracks`), or `null` when nothing is copied. */
export function clipboardOf(
  projectId: string,
  items: readonly ItemEntity[],
  tracks: readonly TrackEntity[],
): Clipboard | null {
  const numbers = trackNumbers(tracks);
  const kinds = new Map(tracks.map((track) => [track.id, track.kind]));
  const entries = items.flatMap(({ trackId, ...item }) => {
    const kind = kinds.get(trackId);
    const number = numbers.get(trackId);
    return kind === undefined || number === undefined ? [] : [{ kind, number, item }];
  });
  return entries.length === 0 ? null : { projectId, entries };
}

/**
 * The track-number shift of a paste for each kind: the active track receives the copies of its
 * kind that were on the lowest-numbered track and the others keep their distances; each kind's
 * shift is then reduced so its copies find tracks (picture copies go where asked even when the
 * sequence has fewer audio tracks).
 */
function pasteOffsets(
  entries: readonly ClipboardEntry[],
  tracks: readonly TrackEntity[],
  active: TrackEntity | null,
): Readonly<Record<TrackKind, number>> {
  const byKind = tracksByKind(tracks);
  const sameKind = entries.filter((entry) => entry.kind === active?.kind);
  const wanted =
    active === null || sameKind.length === 0
      ? 0
      : (trackNumbers(tracks).get(active.id) ?? 0) -
        Math.min(...sameKind.map((entry) => entry.number));
  const offset = (kind: TrackKind) => {
    const numbers = entries.filter((entry) => entry.kind === kind).map((entry) => entry.number);
    if (numbers.length === 0) return 0;
    const low = -Math.min(...numbers);
    const high = byKind[kind].length - 1 - Math.max(...numbers);
    return low > high ? 0 : Math.min(high, Math.max(low, wanted));
  };
  return { video: offset('video'), audio: offset('audio') };
}

/**
 * The copies to paste at `at` (the earliest copy starts there, the others keep their offsets) on
 * the tracks chosen from the active track; copies left without a track are dropped.
 */
export function pastedItems(
  clipboard: Clipboard,
  tracks: readonly TrackEntity[],
  at: Flicks,
  active: TrackEntity | null,
): PastedItem[] {
  const byKind = tracksByKind(tracks);
  const offsets = pasteOffsets(clipboard.entries, tracks, active);
  const earliest = Math.min(...clipboard.entries.map((entry) => entry.item.timelineRange.start));
  return clipboard.entries.flatMap(({ kind, number, item }) => {
    const track = byKind[kind][number + offsets[kind]];
    if (track === undefined) return [];
    const start = (at + item.timelineRange.start - earliest) as Flicks;
    return [
      { trackId: track.id, item: { ...item, timelineRange: { ...item.timelineRange, start } } },
    ];
  });
}

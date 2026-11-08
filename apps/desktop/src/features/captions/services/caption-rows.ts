import type {
  CaptionStyle,
  CaptionStyleId,
  CaptionWord,
  Flicks,
  ItemEntity,
  ItemId,
  SequenceEntity,
  TrackEntity,
  TrackId,
} from '@se/protocol';
import { captionText } from '@se/engine';

/** One caption of the list. */
export interface CaptionRow {
  readonly itemId: ItemId;
  readonly trackId: TrackId;
  readonly start: Flicks;
  readonly end: Flicks;
  readonly words: readonly CaptionWord[];
  readonly text: string;
  readonly styleId: CaptionStyleId;
}

/** The captions of `sequence` (every track), in time order then track order. */
export function captionRows(
  sequence: SequenceEntity,
  tracks: Readonly<Record<TrackId, TrackEntity>>,
  items: Readonly<Record<ItemId, ItemEntity>>,
): CaptionRow[] {
  const rows: CaptionRow[] = [];
  for (const trackId of sequence.trackIds) {
    const track = tracks[trackId];
    if (track === undefined) continue;
    for (const itemId of track.itemIds) {
      const item = items[itemId];
      if (item?.kind.kind !== 'caption') continue;
      const { start, duration } = item.timelineRange;
      rows.push({
        itemId,
        trackId,
        start,
        end: (start + duration) as Flicks,
        words: item.kind.words,
        text: captionText(item.kind),
        styleId: item.kind.styleId,
      });
    }
  }
  return rows.sort((left, right) => left.start - right.start);
}

/** End of the last item of `sequence` (where its content ends), zero when empty. */
export function contentEnd(
  sequence: SequenceEntity,
  tracks: Readonly<Record<TrackId, TrackEntity>>,
  items: Readonly<Record<ItemId, ItemEntity>>,
): Flicks {
  let end = 0;
  for (const trackId of sequence.trackIds) {
    for (const itemId of tracks[trackId]?.itemIds ?? []) {
      const range = items[itemId]?.timelineRange;
      if (range !== undefined) end = Math.max(end, range.start + range.duration);
    }
  }
  return end as Flicks;
}

/** Index of the row playing at `time` (half-open ranges), or −1. */
export function activeRowIndex(rows: readonly CaptionRow[], time: Flicks): number {
  return rows.findIndex((row) => time >= row.start && time < row.end);
}

/** Who a style change applies to: the selected captions, or every caption of their track. */
export type StyleScope = 'selection' | 'track';

/**
 * The captions a style change targets: with `selection`, the selected captions (else the one
 * under the playhead); with `track`, every caption of the track of the first of them (else of
 * the first caption track).
 */
export function styleTargets(
  rows: readonly CaptionRow[],
  scope: StyleScope,
  selected: readonly ItemId[],
  activeIndex: number,
): ItemId[] {
  const chosen = rows.filter((row) => selected.includes(row.itemId));
  const fallback = rows[activeIndex] ?? rows[0];
  const anchor = chosen[0] ?? fallback;
  if (anchor === undefined) return [];
  if (scope === 'track') {
    return rows.filter((row) => row.trackId === anchor.trackId).map((row) => row.itemId);
  }
  return chosen.length > 0 ? chosen.map((row) => row.itemId) : [anchor.itemId];
}

/** The style of `itemId`'s caption among `styles`. */
export function styleOfRow(
  row: CaptionRow | undefined,
  styles: readonly CaptionStyle[],
): CaptionStyle | undefined {
  return row === undefined ? undefined : styles.find((style) => style.id === row.styleId);
}

/** The rows of `trackId`, in time order. */
export function rowsOfTrack(rows: readonly CaptionRow[], trackId: TrackId): CaptionRow[] {
  return rows.filter((row) => row.trackId === trackId);
}

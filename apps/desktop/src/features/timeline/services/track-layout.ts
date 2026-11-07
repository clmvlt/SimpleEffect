import type { TrackEntity, TrackId, TrackKind } from '@se/protocol';

/** A track as displayed: its row's position in the track area. */
export interface TrackRow {
  readonly track: TrackEntity;
  /** Top of the row in pixels from the top of the track area. */
  readonly top: number;
  readonly height: number;
}

/** The rows of the track area: video tracks above the divider, audio tracks below. */
export interface TrackLayout {
  readonly video: readonly TrackRow[];
  readonly audio: readonly TrackRow[];
  /** Top of the divider between video and audio tracks. */
  readonly dividerTop: number;
  /** Height of the whole track area. */
  readonly height: number;
}

/** Height of the divider between the video and audio sections. */
export const SECTION_DIVIDER_HEIGHT = 6;

/**
 * The display order of a sequence's tracks: video tracks stack in document order (first =
 * bottom), so they are listed top-down in reverse; audio tracks are listed in document order
 * (A1 first) below them, as in Premiere.
 */
export function layoutTracks(
  tracks: readonly TrackEntity[],
  heightOf: (track: TrackEntity) => number,
): TrackLayout {
  let top = 0;
  const place = (track: TrackEntity): TrackRow => {
    const row = { track, top, height: heightOf(track) };
    top += row.height;
    return row;
  };
  const video = tracks
    .filter((track) => track.kind === 'video')
    .reverse()
    .map(place);
  const dividerTop = top;
  top += SECTION_DIVIDER_HEIGHT;
  const audio = tracks.filter((track) => track.kind === 'audio').map(place);
  return { video, audio, dividerTop, height: top };
}

/** Every row of a layout, top to bottom. */
export function allRows(layout: TrackLayout): readonly TrackRow[] {
  return [...layout.video, ...layout.audio];
}

/** The row under `y` (pixels from the top of the track area), if any. */
export function rowAt(layout: TrackLayout, y: number): TrackRow | undefined {
  return allRows(layout).find((row) => y >= row.top && y < row.top + row.height);
}

/** Where a new track goes relative to an existing one, as seen on screen. */
export type TrackPlacement = 'above' | 'below';

/**
 * The document index for a track added `placement` an existing track (`addTrack`'s `index`):
 * on screen « above » a video track is later in the list (it stacks higher), « above » an audio
 * track is earlier.
 */
export function insertionIndex(
  trackIds: readonly TrackId[],
  target: TrackEntity,
  placement: TrackPlacement,
): number {
  const index = trackIds.indexOf(target.id);
  const after = target.kind === 'video' ? placement === 'above' : placement === 'below';
  return after ? index + 1 : index;
}

/**
 * The full track order after moving `moved` to display position `to` among the tracks of its
 * kind (top-down). Tracks of the other kind keep their places in the list.
 */
export function reorderedTrackIds(
  trackIds: readonly TrackId[],
  kindOf: (id: TrackId) => TrackKind | undefined,
  moved: TrackId,
  to: number,
): TrackId[] {
  const kind = kindOf(moved);
  if (kind === undefined) return [...trackIds];
  const sameKind = trackIds.filter((id) => kindOf(id) === kind);
  const display = kind === 'video' ? [...sameKind].reverse() : sameKind;
  const without = display.filter((id) => id !== moved);
  const clamped = Math.max(0, Math.min(to, without.length));
  const reordered = [...without.slice(0, clamped), moved, ...without.slice(clamped)];
  const documentOrder = kind === 'video' ? reordered.reverse() : reordered;
  let next = 0;
  return trackIds.map((id) => (kindOf(id) === kind ? (documentOrder[next++] ?? id) : id));
}

/**
 * The display position (among the other tracks of its section) where a track dragged to `y`
 * lands: the number of the section's other rows whose middle is above `y`.
 */
export function dropIndex(section: readonly TrackRow[], moved: TrackId, y: number): number {
  return section.filter((row) => row.track.id !== moved && row.top + row.height / 2 < y).length;
}

/** Top of the line showing where a dragged track would land (`index` from {@link dropIndex}). */
export function dropLineTop(section: readonly TrackRow[], moved: TrackId, index: number): number {
  const others = section.filter((row) => row.track.id !== moved);
  const before = others[index - 1];
  if (before !== undefined) return before.top + before.height;
  return others[0]?.top ?? section[0]?.top ?? 0;
}

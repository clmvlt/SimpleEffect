import type { Flicks, ItemEntity, TrackId } from '@se/protocol';

import { snapToFrame } from '../../../../shared/time';
import { snapPoints, SNAP_PIXELS, snapShift, type Snap } from '../../services/snapping';
import type { TrackLayout } from '../../services/track-layout';
import { pixelsToDuration, type Viewport } from '../../services/viewport';
import type { TimelineCommands } from '../use-timeline-commands';
import type { TimelineModel } from '../use-timeline-model';

/** What a lane gesture reads when it starts and while it runs (always the latest values). */
export interface GestureContext {
  readonly model: TimelineModel;
  readonly layout: TrackLayout;
  readonly viewport: Viewport;
  readonly commands: TimelineCommands;
  /** Whether dragged edges snap (`N`, toolbar magnet): read on every move. */
  readonly snapping: boolean;
  readonly playhead: Flicks;
  /** The in and out points that are set. */
  readonly inOut: readonly Flicks[];
}

/** A point in the lanes, in pixels from their top-left corner. */
export interface LanePoint {
  readonly x: number;
  readonly y: number;
}

/** Modifier keys of a gesture, read on every move and at the release. */
export interface GestureModifiers {
  /** Ctrl (⌘ on macOS): insert instead of overwrite, ripple trims and gap edits. */
  readonly insert: boolean;
  /** Alt: copy instead of move. */
  readonly alt: boolean;
}

/** A gesture in the lanes, driven by the pointer once it started. */
export interface LaneGesture {
  readonly move: (point: LanePoint, modifiers: GestureModifiers) => void;
  /** The pointer was released: send the command (the preview stays until its patch). */
  readonly finish: (point: LanePoint, modifiers: GestureModifiers) => Promise<void>;
  /** Escape or a lost pointer: drop the preview, send nothing. */
  readonly cancel: () => void;
}

/** A pointer movement of `pixels` as whole frames of the sequence (the grid every edit uses). */
export function frameDelta(context: GestureContext, pixels: number): Flicks {
  const raw = pixelsToDuration(pixels, context.viewport);
  return snapToFrame(raw, context.model.sequence.format.frameRate, 'nearest');
}

/** The ids of tracks that are locked (never edited). */
export function lockedTrackIds(context: GestureContext): ReadonlySet<TrackId> {
  return new Set(context.model.tracks.filter((track) => track.locked).map((track) => track.id));
}

/**
 * The snap of `edges` when snapping is on: to the playhead, item edges (except `exclude`),
 * markers and in/out points within {@link SNAP_PIXELS}.
 */
export function snapEdges(
  context: GestureContext,
  edges: readonly Flicks[],
  exclude: readonly ItemEntity[],
): Snap | null {
  if (!context.snapping) return null;
  const points = snapPoints({
    items: context.model.items,
    exclude: new Set(exclude.map((item) => item.id)),
    playhead: context.playhead,
    markers: context.model.markers,
    inOut: context.inOut,
  });
  return snapShift(edges, points, pixelsToDuration(SNAP_PIXELS, context.viewport));
}

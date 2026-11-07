import type { Animatable, Flicks, ItemEntity } from '@se/protocol';

import { snapToFrame } from '../../../../shared/time';
import { fadeLength, mediaClipOf, type FadeEdge } from '../../services/audio-edits';
import { clipEditPreview } from '../../services/preview';
import { allRows } from '../../services/track-layout';
import { pixelsToDuration, xToTime } from '../../services/viewport';
import {
  VOLUME_BOTTOM_DB,
  VOLUME_TOP_DB,
  fractionToVolume,
  moveVolumeKeyframe,
  shiftVolume,
  volumeBand,
} from '../../services/volume-line';
import { clearGesturePreview, setGesturePreview } from '../../store/gesture-store';
import type { GestureContext, LaneGesture, LanePoint } from './gesture-context';

/**
 * Dragging the fade handle at the `edge` corner of a media clip: the fade lengthens towards the
 * middle of the clip, in whole frames. One `updateItems` command on release.
 */
export function startFadeGesture(
  context: () => GestureContext,
  item: ItemEntity,
  edge: FadeEdge,
  origin: LanePoint,
): LaneGesture {
  const clip = mediaClipOf(item);
  const initial: Flicks =
    clip === null ? (0 as Flicks) : edge === 'in' ? clip.fadeIn : clip.fadeOut;
  let length = initial;
  const move = (point: LanePoint) => {
    const current = context();
    const delta = pixelsToDuration(point.x - origin.x, current.viewport);
    length = fadeLength(item, edge, delta, current.model.sequence.format.frameRate);
    setGesturePreview(
      clipEditPreview(item.id, edge === 'in' ? { fadeIn: length } : { fadeOut: length }),
    );
  };
  const finish = async (point: LanePoint) => {
    move(point);
    if (length !== initial) {
      const fade = edge === 'in' ? { fadeIn: length } : { fadeOut: length };
      await context().commands.updateItems([
        { itemId: item.id, patch: { kind: 'mediaClip', ...fade } },
      ]);
    }
    clearGesturePreview();
  };
  return { move, finish, cancel: clearGesturePreview };
}

/** Height in pixels of the volume band of the row holding `item`. */
function bandOf(context: GestureContext, item: ItemEntity) {
  const row = allRows(context.layout).find((candidate) => candidate.track.id === item.trackId);
  const band = volumeBand(row?.height ?? 0);
  return { top: (row?.top ?? 0) + band.top, height: Math.max(1, band.height) };
}

/** Sends `volume` as the clip's volume (one undo step) and drops the preview. */
async function commitVolume(
  context: () => GestureContext,
  item: ItemEntity,
  volume: Animatable<number>,
): Promise<void> {
  await context().commands.setItemAnimation(item.id, { property: 'volumeDb', value: volume });
  clearGesturePreview();
}

/**
 * Dragging the volume line of an audio clip up or down: every keyframe (or the static volume)
 * moves by the same number of decibels. One `setItemAnimation` command on release.
 */
export function startVolumeShiftGesture(
  context: () => GestureContext,
  item: ItemEntity,
  origin: LanePoint,
): LaneGesture {
  const initial = mediaClipOf(item)?.volumeDb ?? { kind: 'static', value: 0 };
  let volume = initial;
  const move = (point: LanePoint) => {
    const { height } = bandOf(context(), item);
    const deltaDb = ((origin.y - point.y) / height) * (VOLUME_TOP_DB - VOLUME_BOTTOM_DB);
    volume = shiftVolume(initial, Math.round(deltaDb * 10) / 10);
    setGesturePreview(clipEditPreview(item.id, { volumeDb: volume }));
  };
  const finish = async (point: LanePoint) => {
    move(point);
    if (volume === initial) clearGesturePreview();
    else await commitVolume(context, item, volume);
  };
  return { move, finish, cancel: clearGesturePreview };
}

/**
 * Dragging keyframe `index` of an audio clip's volume curve: to another instant (whole frames,
 * between its neighbours) and another volume. One `setItemAnimation` command on release.
 */
export function startVolumeKeyframeGesture(
  context: () => GestureContext,
  item: ItemEntity,
  index: number,
): LaneGesture {
  const initial = mediaClipOf(item)?.volumeDb ?? { kind: 'static', value: 0 };
  let volume = initial;
  const move = (point: LanePoint) => {
    const current = context();
    const band = bandOf(current, item);
    const local = (xToTime(point.x, current.viewport) - item.timelineRange.start) as Flicks;
    const time = snapToFrame(local, current.model.sequence.format.frameRate, 'nearest');
    const db = fractionToVolume((band.top + band.height - point.y) / band.height);
    volume = moveVolumeKeyframe(initial, index, time, db, item.timelineRange.duration);
    setGesturePreview(clipEditPreview(item.id, { volumeDb: volume }));
  };
  const finish = async (point: LanePoint) => {
    move(point);
    if (volume === initial) clearGesturePreview();
    else await commitVolume(context, item, volume);
  };
  return { move, finish, cancel: clearGesturePreview };
}

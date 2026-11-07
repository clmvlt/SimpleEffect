import type { Flicks, ItemEntity } from '@se/protocol';
import type { PointerEvent } from 'react';

import { snapToFrame } from '../../../shared/time';
import { mediaClipOf, openCuts, transitionMarks, transitionOn } from '../services/audio-edits';
import { xToTime } from '../services/viewport';
import { addVolumeKeyframe, removeVolumeKeyframe } from '../services/volume-line';
import { selectTransition } from '../store/selection-store';
import type { GestureContext, LaneGesture, LanePoint } from './gestures/gesture-context';
import {
  startFadeGesture,
  startVolumeKeyframeGesture,
  startVolumeShiftGesture,
} from './gestures/sound-gestures';
import {
  startTransitionCreateGesture,
  startTransitionResizeGesture,
} from './gestures/transition-gesture';
import { defaultTransitionDuration } from './use-transition-actions';

/** A press on a sound control: the gesture a drag starts, and what a plain click does. */
export interface SoundPress {
  readonly start: () => LaneGesture | null;
  readonly click: () => void;
}

const nothing = () => null;

/** The element of `selector` around the event's target, if any. */
function around(event: PointerEvent<HTMLElement>, selector: string): Element | null {
  return event.target instanceof Element ? event.target.closest(selector) : null;
}

/**
 * The value of attribute `name` of the element of `selector` around the event's target
 * (attributes rather than `dataset`: some targets are SVG elements).
 */
function attributeAround(event: PointerEvent<HTMLElement>, selector: string, name: string) {
  return around(event, selector)?.getAttribute(name) ?? null;
}

/** Sends `volume` as the clip's volume. */
function setVolume(
  context: () => GestureContext,
  item: ItemEntity,
  volume = mediaClipOf(item)?.volumeDb,
) {
  if (volume !== undefined) {
    void context().commands.setItemAnimation(item.id, { property: 'volumeDb', value: volume });
  }
}

/** A press on a transition (its body selects it, its edges resize it). */
function transitionPress(context: () => GestureContext, element: Element, edge: string | null) {
  const { model } = context();
  const track = model.tracks.find(
    (candidate) => candidate.id === element.getAttribute('data-track-id'),
  );
  if (track === undefined || track.locked) return null;
  const marks = transitionMarks(track, model.itemsByTrack.get(track.id) ?? []);
  const mark = marks.find(
    (candidate) => candidate.transition.id === element.getAttribute('data-transition-id'),
  );
  if (mark === undefined) return null;
  const select = () => {
    selectTransition(mark.transition.id, track.id);
  };
  const start =
    edge === 'start' || edge === 'end'
      ? () => startTransitionResizeGesture(context, track, mark, edge)
      : nothing;
  return { start, click: select };
}

/** A press on the handle of a cut: a drag sizes a new cross-fade, a click adds the default one. */
function cutPress(context: () => GestureContext, element: Element) {
  const { model } = context();
  const track = model.tracks.find(
    (candidate) => candidate.id === element.getAttribute('data-track-id'),
  );
  if (track === undefined || track.locked) return null;
  const cut = openCuts(track, model.itemsByTrack.get(track.id) ?? []).find(
    (candidate) => candidate.left.id === element.getAttribute('data-cut-left'),
  );
  if (cut === undefined) return null;
  return {
    start: () => startTransitionCreateGesture(context, track, cut),
    click: () => {
      void context().commands.addTransitions([
        transitionOn(track, cut, defaultTransitionDuration()),
      ]);
    },
  };
}

/** A press on the fade handles or the volume curve of `item`. */
function clipPress(
  context: () => GestureContext,
  event: PointerEvent<HTMLElement>,
  item: ItemEntity,
  origin: LanePoint,
  insert: boolean,
): SoundPress | null {
  const fade = attributeAround(event, '[data-fade-handle]', 'data-fade-handle');
  if (fade === 'in' || fade === 'out') {
    return { start: () => startFadeGesture(context, item, fade, origin), click: () => undefined };
  }
  const volume = mediaClipOf(item)?.volumeDb;
  const keyframe = attributeAround(event, '[data-volume-keyframe]', 'data-volume-keyframe');
  if (keyframe !== null && volume !== undefined) {
    const index = Number(keyframe);
    return {
      start: () => startVolumeKeyframeGesture(context, item, index),
      click: () => {
        if (event.altKey) setVolume(context, item, removeVolumeKeyframe(volume, index));
      },
    };
  }
  if (around(event, '[data-volume-line]') !== null && volume !== undefined) {
    return {
      start: () => startVolumeShiftGesture(context, item, origin),
      click: () => {
        if (!insert) return;
        const current = context();
        const local = (xToTime(origin.x, current.viewport) - item.timelineRange.start) as Flicks;
        const time = snapToFrame(local, current.model.sequence.format.frameRate, 'nearest');
        const clamped = Math.min(item.timelineRange.duration, Math.max(0, time)) as Flicks;
        setVolume(context, item, addVolumeKeyframe(volume, clamped));
      },
    };
  }
  return null;
}

/**
 * The sound or transition control under a press in the lanes — a transition or its edge, a cut
 * handle, a fade handle, the volume line or one of its keyframes — or `null`. `insert` is
 * Ctrl (⌘ on macOS): a click on the volume line then adds a keyframe; Alt+click removes one.
 */
export function soundPress(
  context: () => GestureContext,
  event: PointerEvent<HTMLElement>,
  item: ItemEntity | undefined,
  origin: LanePoint,
  insert: boolean,
): SoundPress | null {
  const transition = around(event, '[data-transition-id]');
  if (transition !== null) {
    const edge = attributeAround(event, '[data-transition-edge]', 'data-transition-edge');
    return transitionPress(context, transition, edge);
  }
  const cut = around(event, '[data-cut-left]');
  if (cut !== null) return cutPress(context, cut);
  return item === undefined ? null : clipPress(context, event, item, origin, insert);
}

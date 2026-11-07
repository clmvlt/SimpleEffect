import { useStore } from 'zustand';

import { useShortcutAction } from '../../../shared/shortcuts';
import { clearSelection, selectItems } from '../store/selection-store';
import { setSnapping, timelineViewStore } from '../store/timeline-view-store';
import type { InOutControls } from './use-in-out';
import type { PlayheadControls } from './use-playhead-controls';
import type { TimelineCommands } from './use-timeline-commands';
import type { TimelineModel } from './use-timeline-model';
import { ZOOM_STEP, type TimelineViewportApi } from './use-timeline-viewport';

/** What the timeline shortcuts act on. */
export interface TimelineShortcutTargets {
  readonly model: TimelineModel;
  readonly viewport: TimelineViewportApi;
  readonly playhead: PlayheadControls;
  readonly inOut: InOutControls;
  readonly commands: TimelineCommands;
}

/**
 * Binds the timeline's named shortcut actions (live while the timeline region is active):
 * playhead navigation, in/out points, markers, zoom, selection and snapping.
 */
export function useTimelineShortcuts({
  model,
  viewport,
  playhead,
  inOut,
  commands,
}: TimelineShortcutTargets): void {
  const snapping = useStore(timelineViewStore, (state) => state.snapping);
  useShortcutAction('goToStart', playhead.goToStart);
  useShortcutAction('goToEnd', playhead.goToEnd);
  useShortcutAction('previousFrame', () => {
    playhead.step(-1);
  });
  useShortcutAction('nextFrame', () => {
    playhead.step(1);
  });
  useShortcutAction('backTenFrames', () => {
    playhead.step(-10);
  });
  useShortcutAction('forwardTenFrames', () => {
    playhead.step(10);
  });
  useShortcutAction('previousEditPoint', playhead.previousEditPoint);
  useShortcutAction('nextEditPoint', playhead.nextEditPoint);
  useShortcutAction('markIn', () => {
    inOut.markIn(playhead.time());
  });
  useShortcutAction('markOut', () => {
    inOut.markOut(playhead.time());
  });
  useShortcutAction('clearInOut', inOut.clear);
  useShortcutAction('addMarker', () => {
    void commands.addMarker(model.sequence.id, playhead.time(), '', null);
  });
  useShortcutAction('zoomIn', () => {
    viewport.zoomBy(ZOOM_STEP);
  });
  useShortcutAction('zoomOut', () => {
    viewport.zoomBy(1 / ZOOM_STEP);
  });
  useShortcutAction('zoomToFit', viewport.fit);
  useShortcutAction('selectAll', () => {
    selectItems(model.items.map((item) => item.id));
  });
  useShortcutAction('deselectAll', clearSelection);
  useShortcutAction('toggleSnapping', () => {
    setSnapping(!snapping);
  });
}

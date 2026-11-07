import { useState, type MouseEvent } from 'react';

import { snapToFrame } from '../../../shared/time';
import type { LaneMenuTarget } from '../components/LaneMenu';
import { gapAt } from '../services/gaps';
import { linkedItems } from '../services/selection';
import { rowAt } from '../services/track-layout';
import { xToTime } from '../services/viewport';
import { selectGap, selectionStore, selectItems, setActiveTrack } from '../store/selection-store';
import type { GestureContext } from './gestures/gesture-context';

/**
 * The target of the lanes' context menu, found on right click: an item (selected with its
 * linked partners unless it already is), a gap (selected), or empty space.
 */
export function useLaneMenu(context: () => GestureContext) {
  const [target, setTarget] = useState<LaneMenuTarget>({ kind: 'empty' });

  const onContextMenu = (event: MouseEvent<HTMLDivElement>) => {
    const { model, layout, viewport } = context();
    const bounds = event.currentTarget.getBoundingClientRect();
    const [x, y] = [event.clientX - bounds.left, event.clientY - bounds.top];
    const time = snapToFrame(xToTime(x, viewport), model.sequence.format.frameRate, 'nearest');
    const element = event.target instanceof Element ? event.target.closest('[data-item-id]') : null;
    const itemId = element instanceof HTMLElement ? element.dataset['itemId'] : undefined;
    const item = itemId === undefined ? undefined : model.itemsById[itemId];
    if (item !== undefined) {
      if (!selectionStore.getState().itemIds.includes(item.id)) {
        const ids = model.items.map((candidate) => candidate.id);
        selectItems(linkedItems(item, model.itemsById, ids, false));
      }
      setActiveTrack(item.trackId);
      setTarget({ kind: 'item', item, time });
      return;
    }
    const row = rowAt(layout, y);
    const items = row === undefined ? [] : (model.itemsByTrack.get(row.track.id) ?? []);
    const gap = row === undefined || row.track.locked ? null : gapAt(items, row.track.id, time);
    if (gap !== null) {
      selectGap(gap);
      setTarget({ kind: 'gap', gap });
      return;
    }
    if (row !== undefined) setActiveTrack(row.track.id);
    setTarget({ kind: 'empty' });
  };

  return { target, onContextMenu };
}

/**
 * @vitest-environment jsdom
 */
import { act, createEvent, fireEvent, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it } from 'vitest';

import { ASSET_DRAG_TYPE, TEMPLATE_DRAG_TYPE } from '../../../shared/drag';
import { gestureStore } from '../store/gesture-store';
import { selectionStore } from '../store/selection-store';
import {
  itemElement,
  renderTimeline as render,
  resetTimelineStores,
  seconds,
} from '../testing/render-timeline';

const clients: { close: () => void }[] = [];

beforeEach(resetTimelineStores);

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
});

function renderTimeline() {
  const rendered = render();
  clients.push(rendered.client);
  return rendered;
}

/** Presses at `from`, drags to `to` (lane pixels, x then y) and releases there. */
function drag(
  target: HTMLElement,
  lanes: HTMLElement,
  from: readonly [number, number],
  to: readonly [number, number],
  modifiers: Record<string, boolean> = {},
) {
  fireEvent.pointerDown(target, { button: 0, clientX: from[0], clientY: from[1], ...modifiers });
  fireEvent.pointerMove(lanes, { clientX: to[0], clientY: to[1], ...modifiers });
  fireEvent.pointerUp(lanes, { clientX: to[0], clientY: to[1], ...modifiers });
}

/** The trim handle on `edge` of item `id`. */
function trimHandle(id: string, edge: 'start' | 'end'): HTMLElement {
  const handle = itemElement(id).querySelector<HTMLElement>(`[data-trim-edge="${edge}"]`);
  if (handle === null) throw new Error(`${id} shows its ${edge} handle`);
  return handle;
}

/** Fires a drag event at lane pixels (x, y): jsdom has no `DragEvent`, so the position is set. */
function fireDrag(
  kind: 'dragOver' | 'drop',
  lanes: HTMLElement,
  x: number,
  y: number,
  data: object,
) {
  const event = createEvent[kind](lanes, { dataTransfer: data });
  Object.defineProperties(event, { clientX: { value: x }, clientY: { value: y } });
  fireEvent(lanes, event);
}

/** Waits until the gesture's command answered and its preview went away. */
async function settled() {
  await waitFor(() => {
    expect(gestureStore.getState().preview).toBeNull();
  });
}

/** Clicks empty lane space at lane pixels (x, y). */
function clickLanes(lanes: HTMLElement, x: number, y: number) {
  fireEvent.pointerDown(lanes, { button: 0, clientX: x, clientY: y });
  fireEvent.pointerUp(lanes, { clientX: x, clientY: y });
}

describe('timeline editing', () => {
  it('moves a clip with one command once released, previewing it meanwhile', async () => {
    const { send, lanes } = renderTimeline();
    fireEvent.pointerDown(itemElement('c2'), { button: 0, clientX: 110, clientY: 70 });
    fireEvent.pointerMove(lanes, { clientX: 160, clientY: 70 });
    expect(document.querySelector('[data-preview-clip="placement"]')).toBeInTheDocument();
    expect(send).not.toHaveBeenCalledWith(expect.objectContaining({ kind: 'moveItems' }));
    fireEvent.pointerUp(lanes, { clientX: 160, clientY: 70 });
    expect(send).toHaveBeenCalledWith({
      kind: 'moveItems',
      projectId: 'project-1',
      moves: [{ itemId: 'c2', trackId: 'v1', start: seconds(3) }],
      mode: 'overwrite',
    });
    await settled();
  });

  it('moves linked items together, across tracks, and inserts with Ctrl', () => {
    const { send, lanes } = renderTimeline();
    // The caption and its linked sound move together, one second later, in insert mode.
    drag(itemElement('c1'), lanes, [10, 70], [60, 70], { ctrlKey: true });
    expect(send).toHaveBeenCalledWith({
      kind: 'moveItems',
      projectId: 'project-1',
      moves: [
        { itemId: 'c1', trackId: 'v1', start: seconds(1) },
        { itemId: 's', trackId: 'a1', start: seconds(1) },
      ],
      mode: 'insert',
    });
  });

  it('copies with Alt and cancels a drag with Escape', () => {
    const { send, lanes } = renderTimeline();
    drag(itemElement('c2'), lanes, [110, 70], [160, 20], { altKey: true });
    expect(send).toHaveBeenCalledWith({
      kind: 'duplicateItems',
      projectId: 'project-1',
      copies: [{ itemId: 'c2', trackId: 'v2', start: seconds(3) }],
      mode: 'overwrite',
    });
    send.mockClear();
    fireEvent.pointerDown(itemElement('c2'), { button: 0, clientX: 110, clientY: 70 });
    fireEvent.pointerMove(lanes, { clientX: 160, clientY: 70 });
    fireEvent.keyDown(window, { key: 'Escape' });
    expect(gestureStore.getState().preview).toBeNull();
    fireEvent.pointerUp(lanes, { clientX: 160, clientY: 70 });
    expect(send).not.toHaveBeenCalled();
  });

  it('trims an edge, and ripple trims a clip with its sound with Ctrl', () => {
    const { send, lanes } = renderTimeline();
    drag(trimHandle('c2', 'end'), lanes, [150, 70], [200, 70]);
    expect(send).toHaveBeenLastCalledWith({
      kind: 'trimItems',
      projectId: 'project-1',
      trims: [{ itemId: 'c2', edge: 'end', to: seconds(4) }],
      ripple: false,
    });
    // Ten frames earlier (20 px at 50 px per second).
    drag(trimHandle('c1', 'end'), lanes, [50, 70], [30, 70], { ctrlKey: true });
    expect(send).toHaveBeenLastCalledWith({
      kind: 'trimItems',
      projectId: 'project-1',
      trims: [
        { itemId: 'c1', edge: 'end', to: seconds(0.6) },
        { itemId: 's', edge: 'end', to: seconds(0.6) },
      ],
      ripple: true,
    });
  });

  it('cuts at the playhead with S and ripple deletes with Maj+Suppr', async () => {
    const user = userEvent.setup();
    const { send } = renderTimeline();
    await user.keyboard('{Shift>}{ArrowRight}{/Shift}s');
    expect(send).toHaveBeenCalledWith({
      kind: 'splitItems',
      projectId: 'project-1',
      itemIds: ['c1', 's'],
      at: seconds(10 / 25),
    });
    act(() => {
      selectionStore.setState({ itemIds: ['c2'] });
    });
    await user.keyboard('{Shift>}{Delete}{/Shift}');
    expect(send).toHaveBeenLastCalledWith({
      kind: 'deleteItems',
      projectId: 'project-1',
      itemIds: ['c2'],
      ripple: true,
    });
  });

  it('nudges, copies and pastes at the playhead on the active track', async () => {
    const user = userEvent.setup();
    const { send, lanes } = renderTimeline();
    fireEvent.pointerDown(itemElement('c2'), { button: 0, clientX: 110, clientY: 70 });
    fireEvent.pointerUp(lanes, { clientX: 110, clientY: 70 });
    await user.keyboard('.');
    expect(send).toHaveBeenLastCalledWith({
      kind: 'moveItems',
      projectId: 'project-1',
      moves: [{ itemId: 'c2', trackId: 'v1', start: seconds(2 + 1 / 25) }],
      mode: 'overwrite',
    });
    await user.keyboard('{Control>}c{/Control}');
    clickLanes(lanes, 300, 20);
    await user.keyboard('{End}{Control>}v{/Control}');
    const paste = send.mock.calls.at(-1)?.[0];
    if (paste?.kind !== 'pasteItems') throw new Error('Ctrl+V pastes');
    expect(paste.mode).toBe('overwrite');
    expect(paste.items.map(({ trackId, item }) => [trackId, item.id, item.timelineRange])).toEqual([
      ['v2', 'c2', { start: seconds(3), duration: seconds(1) }],
    ]);
  });

  it('selects a gap, drags its handle to lengthen it and closes it with Suppr', async () => {
    const user = userEvent.setup();
    const { send, lanes } = renderTimeline();
    clickLanes(lanes, 75, 70);
    expect(screen.getByRole('status', { name: /^Blanc de/ })).toBeInTheDocument();
    const handle = document.querySelector<HTMLElement>('[data-gap-handle]');
    if (handle === null) throw new Error('the selected gap shows its handle');
    drag(handle, lanes, [100, 70], [150, 70]);
    expect(send).toHaveBeenLastCalledWith({
      kind: 'resizeGap',
      projectId: 'project-1',
      trackId: 'v1',
      start: seconds(1),
      end: seconds(2),
      duration: seconds(2),
    });
    await settled();
    clickLanes(lanes, 75, 70);
    await user.keyboard('{Delete}');
    expect(send).toHaveBeenLastCalledWith({
      kind: 'resizeGap',
      projectId: 'project-1',
      trackId: 'v1',
      start: seconds(1),
      end: seconds(2),
      duration: 0,
    });
  });

  it('places media dropped from the bin on the track under the pointer', () => {
    const { send, lanes } = renderTimeline();
    const dataTransfer = {
      types: [ASSET_DRAG_TYPE],
      getData: () => JSON.stringify(['asset-1']),
      dropEffect: 'none',
    };
    fireDrag('dragOver', lanes, 200, 70, dataTransfer);
    expect(document.querySelectorAll('[data-preview-clip="ghost"]')).toHaveLength(2);
    fireDrag('drop', lanes, 200, 70, dataTransfer);
    expect(send).toHaveBeenCalledWith({
      kind: 'placeMedia',
      projectId: 'project-1',
      sequenceId: 'sequence-1',
      assetIds: ['asset-1'],
      start: seconds(4),
      videoTrack: { kind: 'existing', trackId: 'v1' },
      audioTrack: { kind: 'existing', trackId: 'a1' },
      mode: 'overwrite',
    });
  });

  it('places a template dropped from the browser on the video track under the pointer', async () => {
    const { send, lanes } = renderTimeline();
    const template = { source: 'project', id: 'badge', name: 'Badge', duration: seconds(3) };
    const dataTransfer = {
      types: [TEMPLATE_DRAG_TYPE],
      getData: () => JSON.stringify(template),
      dropEffect: 'none',
    };
    fireDrag('dragOver', lanes, 200, 20, dataTransfer);
    expect(document.querySelectorAll('[data-preview-clip="ghost"]')).toHaveLength(1);
    fireDrag('drop', lanes, 200, 20, dataTransfer);
    expect(send).toHaveBeenCalledWith({
      kind: 'addGraphicItem',
      projectId: 'project-1',
      sequenceId: 'sequence-1',
      at: seconds(4),
      duration: seconds(3),
      graphic: { kind: 'template', templateId: 'badge' },
      trackId: 'v2',
    });
    await settled();
  });

  it('copies a library template dropped on the lanes into the project', async () => {
    const { send, lanes } = renderTimeline();
    const template = { source: 'library', id: 'entry-1', name: 'Bandeau', duration: seconds(5) };
    const dataTransfer = {
      types: [TEMPLATE_DRAG_TYPE],
      getData: () => JSON.stringify(template),
      dropEffect: 'none',
    };
    fireDrag('drop', lanes, 100, 130, dataTransfer);
    expect(send).toHaveBeenCalledWith({
      kind: 'insertLibraryTemplate',
      projectId: 'project-1',
      templateId: 'entry-1',
      sequenceId: 'sequence-1',
      trackId: null,
      start: seconds(2),
    });
    await settled();
  });

  it('offers the item actions in a context menu', async () => {
    const user = userEvent.setup();
    const { send } = renderTimeline();
    fireEvent.contextMenu(itemElement('c2'), { clientX: 120, clientY: 70 });
    expect(selectionStore.getState().itemIds).toEqual(['c2']);
    await user.click(await screen.findByRole('menuitem', { name: 'Supprimer avec ripple' }));
    expect(send).toHaveBeenLastCalledWith({
      kind: 'deleteItems',
      projectId: 'project-1',
      itemIds: ['c2'],
      ripple: true,
    });
  });
});

/**
 * @vitest-environment jsdom
 */
import { act, fireEvent, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { playheadStore } from '../store/playhead-store';
import { selectionStore } from '../store/selection-store';
import { NEW_SEQUENCE_VIEW, timelineViewStore } from '../store/timeline-view-store';
import { seconds } from '../testing/fixtures';
import { renderTimeline as render, resetTimelineStores } from '../testing/render-timeline';

const clients: { close: () => void }[] = [];

beforeEach(resetTimelineStores);

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
});

/** Renders the timeline over the test session; `send` is a spy. */
function renderTimeline() {
  const rendered = render();
  clients.push(rendered.client);
  return rendered;
}

const selectedNames = () =>
  screen
    .getAllByRole('option', { selected: true })
    .map((option) => option.getAttribute('aria-label'));

describe('TimelinePanel', () => {
  it('shows video tracks above audio tracks, newest video track on top', () => {
    renderTimeline();
    const headers = screen.getAllByRole('group', { name: /^Piste / });
    expect(headers.map((header) => header.getAttribute('aria-label'))).toEqual([
      'Piste V2',
      'Piste V1',
      'Piste A1',
    ]);
    expect(within(screen.getByRole('listbox', { name: 'V1' })).getAllByRole('option')).toHaveLength(
      2,
    );
  });

  it('selects by click (with linked items), Maj+click and Ctrl+click', () => {
    renderTimeline();
    const [first, second] = within(screen.getByRole('listbox', { name: 'V1' })).getAllByRole(
      'option',
    );
    if (first === undefined || second === undefined) throw new Error('two captions');
    const click = (element: HTMLElement, modifiers: Record<string, boolean> = {}) => {
      fireEvent.pointerDown(element, { button: 0, ...modifiers });
      fireEvent.pointerUp(element, modifiers);
    };
    click(first);
    expect(selectionStore.getState().itemIds).toEqual(['c1', 's']);
    click(second, { shiftKey: true });
    expect(selectionStore.getState().itemIds).toEqual(['c1', 's', 'c2']);
    click(first, { ctrlKey: true });
    expect(selectionStore.getState().itemIds).toEqual(['c2']);
    click(first, { altKey: true });
    expect(selectedNames()).toEqual(['Bonjour']);
    expect(selectionStore.getState().itemIds).toEqual(['c1']);
  });

  it('selects the items a rectangle touches and deselects on an empty click', () => {
    const { lanes } = renderTimeline();
    vi.spyOn(lanes, 'getBoundingClientRect').mockReturnValue(new DOMRect(0, 0, 1000, 300));
    // Rows: V2 0–52, V1 52–104; 50 px per second before the first fit.
    fireEvent.pointerDown(lanes, { button: 0, clientX: 5, clientY: 10 });
    fireEvent.pointerMove(lanes, { clientX: 140, clientY: 80 });
    expect(document.querySelector('[data-marquee]')).toBeInTheDocument();
    fireEvent.pointerUp(lanes, { clientX: 140, clientY: 80 });
    expect(selectionStore.getState().itemIds).toEqual(['c1', 'c2']);
    fireEvent.pointerDown(lanes, { button: 0, clientX: 900, clientY: 10 });
    fireEvent.pointerUp(lanes, { clientX: 900, clientY: 10 });
    expect(selectionStore.getState().itemIds).toEqual([]);
  });

  it('marks in and out with I and O at the playhead and clears them with Alt+X', async () => {
    const user = userEvent.setup();
    renderTimeline();
    await user.keyboard('i');
    await user.keyboard('{Shift>}{ArrowRight}{/Shift}');
    await user.keyboard('o');
    const view = timelineViewStore.getState().sequences['sequence-1'] ?? NEW_SEQUENCE_VIEW;
    // Ten frames at 25 fps, the out point after the frame under the playhead.
    expect([view.inPoint, view.outPoint]).toEqual([0, seconds(11 / 25)]);
    expect(document.querySelector('[data-in-out-span]')).toBeInTheDocument();
    await user.keyboard('{Alt>}x{/Alt}');
    expect(document.querySelector('[data-in-out-span]')).not.toBeInTheDocument();
  });

  it('adds a marker at the playhead with M', async () => {
    const user = userEvent.setup();
    const { send } = renderTimeline();
    await user.keyboard('{ArrowRight}{ArrowRight}m');
    expect(send).toHaveBeenCalledWith({
      kind: 'addMarker',
      projectId: 'project-1',
      sequenceId: 'sequence-1',
      time: seconds(2 / 25),
      label: '',
      color: null,
    });
  });

  it('renames a track inline and toggles its header buttons, one command each', async () => {
    const user = userEvent.setup();
    const { send } = renderTimeline();
    await user.dblClick(screen.getByText('V1'));
    const field = screen.getByRole('textbox', { name: 'Nom de la piste' });
    await user.clear(field);
    await user.type(field, 'Titres{Enter}');
    expect(send).toHaveBeenCalledWith({
      kind: 'updateTrack',
      projectId: 'project-1',
      trackId: 'v1',
      patch: { name: 'Titres' },
    });
    await user.click(screen.getByRole('button', { name: 'Rendre A1 muette' }));
    expect(send).toHaveBeenLastCalledWith({
      kind: 'updateTrack',
      projectId: 'project-1',
      trackId: 'a1',
      patch: { muted: true },
    });
  });

  it('moves the playhead through edit points with the arrow keys', async () => {
    const user = userEvent.setup();
    renderTimeline();
    await user.keyboard('{ArrowDown}');
    expect(playheadStore.getState().times['sequence-1']).toBe(seconds(1));
    await user.keyboard('{ArrowDown}{ArrowDown}');
    expect(playheadStore.getState().times['sequence-1']).toBe(seconds(3));
    await act(async () => {
      await user.keyboard('{Home}');
    });
    expect(playheadStore.getState().times['sequence-1']).toBe(0);
  });
});

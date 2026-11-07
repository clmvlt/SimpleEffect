/**
 * @vitest-environment jsdom
 */
import { CoreClient, type ProjectSession } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, fireEvent, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { CoreClientProvider } from '../../../shared/core';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { session } from '../../../shared/project-store/testing/session-fixture';
import { ShortcutProvider, ShortcutRegistry } from '../../../shared/shortcuts';
import { seconds, testItem, testSequence, testTrack } from '../testing/fixtures';
import { playheadStore } from '../store/playhead-store';
import { clearSelection, selectionStore } from '../store/selection-store';
import { NEW_SEQUENCE_VIEW, timelineViewStore } from '../store/timeline-view-store';
import { TimelinePanel } from './TimelinePanel';

const clients: CoreClient[] = [];

beforeEach(() => {
  window.localStorage.clear();
  timelineViewStore.setState({
    activeSequenceId: null,
    sequences: {},
    trackHeights: {},
    snapping: true,
  });
  playheadStore.setState({ times: {} });
  clearSelection();
});

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
});

/** A session with tracks V2, V1 (video) and A1 (audio): linked clip p/s, captions c1, c2. */
function timelineSession(): ProjectSession {
  const base = session();
  return {
    ...base,
    snapshot: {
      ...base.snapshot,
      sequences: [testSequence(['v1', 'v2', 'a1'])],
      tracks: [
        testTrack('v1', 'video', ['c1', 'c2']),
        testTrack('v2', 'video'),
        testTrack('a1', 'audio', ['s']),
      ],
      items: [
        testItem('c1', 'v1', 0, 1, { linkGroup: 'g' }),
        testItem('c2', 'v1', 2, 1),
        testItem('s', 'a1', 0, 1, { linkGroup: 'g' }),
      ],
      markers: [],
    },
  };
}

/** Renders the timeline over the session; the core never connects and `send` is a spy. */
function renderTimeline() {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi.spyOn(client, 'send').mockResolvedValue({ kind: 'updateTrack' } as never);
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(timelineSession());
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <ShortcutProvider registry={new ShortcutRegistry('other')}>
          <ProjectStoreProvider store={store} sync={sync}>
            <TooltipProvider>
              <TimelinePanel />
            </TooltipProvider>
          </ProjectStoreProvider>
        </ShortcutProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return { send };
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
    fireEvent.pointerDown(first, { button: 0 });
    expect(selectionStore.getState().itemIds).toEqual(['c1', 's']);
    fireEvent.pointerDown(second, { button: 0, shiftKey: true });
    expect(selectionStore.getState().itemIds).toEqual(['c1', 's', 'c2']);
    fireEvent.pointerDown(first, { button: 0, ctrlKey: true });
    expect(selectionStore.getState().itemIds).toEqual(['c2']);
    fireEvent.pointerDown(first, { button: 0, altKey: true });
    expect(selectedNames()).toEqual(['Bonjour']);
    expect(selectionStore.getState().itemIds).toEqual(['c1']);
  });

  it('selects the items a rectangle touches and deselects on an empty click', () => {
    renderTimeline();
    const lanes = document.querySelector<HTMLElement>('[data-timeline-lanes]');
    if (lanes === null) throw new Error('lanes');
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

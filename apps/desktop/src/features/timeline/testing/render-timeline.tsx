import { CoreClient, type Command, type ProjectSession } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { render } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import { vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { CoreClientProvider } from '../../../shared/core';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { session } from '../../../shared/project-store/testing/session-fixture';
import { ShortcutProvider, ShortcutRegistry } from '../../../shared/shortcuts';
import { TimelinePanel } from '../components/TimelinePanel';
import { gestureStore } from '../store/gesture-store';
import { playheadStore } from '../store/playhead-store';
import { clearSelection } from '../store/selection-store';
import { timelineViewStore } from '../store/timeline-view-store';
import { seconds, testAsset, testItem, testSequence, testTrack } from './fixtures';

/** Resets the timeline stores between tests. */
export function resetTimelineStores(): void {
  window.localStorage.clear();
  timelineViewStore.setState({
    activeSequenceId: null,
    sequences: {},
    trackHeights: {},
    snapping: true,
  });
  playheadStore.setState({ times: {} });
  gestureStore.setState({ preview: null });
  clearSelection();
}

/**
 * A session with tracks V2, V1 (video) and A1 (audio): captions c1 (0–1 s, linked to the sound
 * s on A1) and c2 (2–3 s) on V1, a 30 s video asset with sound.
 */
export function timelineSession(): ProjectSession {
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
      assets: [testAsset('asset-1', '/media/plage.mp4')],
    },
  };
}

/** What every command answers in these tests: its kind and empty ids. */
function answer(command: Command) {
  return { kind: command.kind, itemIds: [], linkGroup: 'g2', trackId: 't', markerId: 'm' };
}

/**
 * Renders the timeline over `snapshot` (default {@link timelineSession}); the core never
 * connects and `send` is a spy answering every command. 50 px per second (jsdom has no layout);
 * rows: V2 0–52 px, V1 52–104, divider, A1 110–162.
 */
export function renderTimeline(snapshot: ProjectSession = timelineSession()) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command: Command) => Promise.resolve(answer(command) as never));
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(snapshot);
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
  const lanes = document.querySelector<HTMLElement>('[data-timeline-lanes]');
  if (lanes === null) throw new Error('the timeline renders its lanes');
  return { send, client, lanes };
}

/** The element of item `id` in the lanes. */
export function itemElement(id: string): HTMLElement {
  const element = document.querySelector<HTMLElement>(`[data-item-id="${id}"]`);
  if (element === null) throw new Error(`item ${id} is rendered`);
  return element;
}

export { seconds };

/**
 * @vitest-environment jsdom
 */
import {
  CoreClient,
  type Command,
  type Flicks,
  type ItemEntity,
  type ProjectSession,
  type TrackEntity,
} from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { CoreClientProvider } from '../../../shared/core';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { session } from '../../../shared/project-store/testing/session-fixture';
import { clearSelection, selectItems } from '../../timeline';
import { InspectorView } from './InspectorView';

const SECOND = 705_600_000 as Flicks;

/** A session with a video track V1 and an audio track A1 holding the clip `m1` (4 s). */
function audioSession(): ProjectSession {
  const base = session();
  const track = (id: string, kind: 'video' | 'audio', itemIds: string[]): TrackEntity => ({
    id,
    sequenceId: 'sequence-1',
    kind,
    name: id,
    muted: false,
    solo: false,
    locked: false,
    hidden: false,
    volumeDb: 0,
    pan: 0,
    itemIds,
    transitions: [],
  });
  return {
    ...base,
    snapshot: {
      ...base.snapshot,
      tracks: [track('v1', 'video', []), track('a1', 'audio', ['m1'])],
      items: [clip()],
    },
  };
}

/** The 4 s audio clip `m1` at −6 dB with a one-second constant-power fade-out. */
function clip(): ItemEntity {
  return {
    trackId: 'a1',
    id: 'm1',
    revision: 1,
    origin: { kind: 'user' },
    timelineRange: { start: 0 as Flicks, duration: (4 * SECOND) as Flicks },
    linkGroup: null,
    kind: {
      kind: 'mediaClip',
      assetId: 'asset-1',
      sourceStart: 0 as Flicks,
      speed: 1,
      transform: null as never,
      volumeDb: { kind: 'static', value: -6 },
      fadeIn: 0 as Flicks,
      fadeOut: SECOND,
      fadeInShape: 'linear',
      fadeOutShape: 'constantPower',
      reframe: { kind: 'fit' },
    },
  };
}

const clients: CoreClient[] = [];
afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  clearSelection();
});

function renderInspector() {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command: Command) => Promise.resolve({ kind: command.kind } as never));
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(audioSession());
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <ProjectStoreProvider store={store} sync={sync}>
          <TooltipProvider>
            <InspectorView />
          </TooltipProvider>
        </ProjectStoreProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return send;
}

describe('the inspector', () => {
  it('stays empty without an audio clip selected', () => {
    renderInspector();
    expect(screen.getByText('Aucune sélection')).toBeInTheDocument();
  });

  it('edits the gain and fades of the selected audio clip, one command per step', async () => {
    const send = renderInspector();
    act(() => {
      selectItems(['m1']);
    });
    const gain = screen.getByRole('spinbutton', { name: 'Gain' });
    expect(gain).toHaveAttribute('aria-valuenow', '-6');
    gain.focus();
    await userEvent.keyboard('{ArrowUp}');
    expect(send).toHaveBeenCalledWith({
      kind: 'setItemAnimation',
      projectId: 'project-1',
      itemId: 'm1',
      animation: { property: 'volumeDb', value: { kind: 'static', value: -5.5 } },
    });
    const fadeIn = screen.getByRole('spinbutton', { name: 'Fondu d’entrée' });
    fadeIn.focus();
    await userEvent.keyboard('{ArrowUp}');
    expect(send).toHaveBeenCalledWith({
      kind: 'updateItems',
      projectId: 'project-1',
      updates: [{ itemId: 'm1', patch: { kind: 'mediaClip', fadeIn: 0.05 * SECOND } }],
    });
  });
});

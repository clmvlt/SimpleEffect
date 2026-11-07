/**
 * @vitest-environment jsdom
 */
import { CoreClient, type AnimationPresetInfo, type Command, type Flicks } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, render, screen, waitFor } from '@testing-library/react';
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
import { SECOND, graphicsSession } from '../../../shared/project-store/testing/graphics-fixture';
import { ShortcutProvider, ShortcutRegion, ShortcutRegistry } from '../../../shared/shortcuts';
import { clearSelection, selectionStore, selectItems, setPlayheadTime } from '../../timeline';
import { animationPresetsStore } from '../store/animation-presets-store';
import { AnimationsPanel } from './AnimationsPanel';
import { GraphicsTools } from './GraphicsTools';

const linear = { kind: 'linear' } as const;
const fade: AnimationPresetInfo = {
  id: 'fade',
  name: 'Fade',
  description: 'Fades in.',
  phases: ['in', 'out'],
  defaultDuration: (SECOND / 2) as Flicks,
  properties: ['opacity'],
  samples: [
    {
      phase: 'in',
      animations: [
        {
          property: 'opacity',
          value: {
            kind: 'keyframed',
            keyframes: [
              { time: 0 as Flicks, value: 0, easing: linear },
              { time: SECOND, value: 1, easing: linear },
            ],
          },
        },
      ],
    },
  ],
};
const typewriter: AnimationPresetInfo = { ...fade, id: 'typewriter', properties: ['reveal'] };

const clients: CoreClient[] = [];
afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  clearSelection();
  animationPresetsStore.setState({ presets: null });
  window.localStorage.clear();
});

function renderTools() {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi.spyOn(client, 'send').mockImplementation((command: Command) => {
    if (command.kind === 'addGraphicItem') {
      return Promise.resolve({
        kind: 'addGraphicItem',
        itemId: 'new-text',
        trackId: 'track-v2',
      } as never);
    }
    if (command.kind === 'listAnimationPresets') {
      return Promise.resolve({
        kind: 'listAnimationPresets',
        presets: [fade, typewriter],
      } as never);
    }
    return Promise.resolve({ kind: command.kind } as never);
  });
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(graphicsSession());
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <ShortcutProvider registry={new ShortcutRegistry('other')}>
          <ProjectStoreProvider store={store} sync={sync}>
            <TooltipProvider>
              <ShortcutRegion context="viewer" claimOnMount>
                <GraphicsTools />
              </ShortcutRegion>
              <AnimationsPanel />
            </TooltipProvider>
          </ProjectStoreProvider>
        </ShortcutProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return send;
}

describe('graphics tools', () => {
  it('add a text at the playhead with T, then select it', async () => {
    const send = renderTools();
    act(() => {
      setPlayheadTime('sequence-1', (2 * SECOND) as Flicks);
    });
    await userEvent.keyboard('t');
    expect(send).toHaveBeenCalledWith({
      kind: 'addGraphicItem',
      projectId: 'project-1',
      sequenceId: 'sequence-1',
      at: 2 * SECOND,
      duration: 5 * SECOND,
      graphic: { kind: 'text', content: 'Titre' },
    });
    await waitFor(() => {
      expect(selectionStore.getState().itemIds).toEqual(['new-text']);
    });
  });

  it('add shapes from their menu', async () => {
    const send = renderTools();
    await userEvent.click(screen.getByRole('button', { name: 'Ajouter une forme' }));
    await userEvent.click(screen.getByRole('menuitem', { name: 'Ellipse' }));
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: 'addGraphicItem',
        graphic: { kind: 'shape', shape: 'ellipse' },
      }),
    );
  });
});

describe('the animations panel', () => {
  it('applies presets the selection can take, in one command', async () => {
    const send = renderTools();
    await screen.findByRole('button', { name: /Fondu/ });
    act(() => {
      selectItems(['title', 'plain']);
    });
    await userEvent.click(screen.getByRole('button', { name: /Fondu/ }));
    expect(send).toHaveBeenCalledWith({
      kind: 'applyAnimationPreset',
      projectId: 'project-1',
      itemIds: ['title', 'plain'],
      preset: 'fade',
      phase: 'in',
      duration: SECOND / 2,
      start: null,
    });
    expect(screen.getByRole('button', { name: /Machine à écrire/ })).toBeEnabled();
  });
});

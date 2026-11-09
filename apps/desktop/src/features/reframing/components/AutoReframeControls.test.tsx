/**
 * @vitest-environment jsdom
 */
import { CoreClient, type Command, type ItemEntity, type VisionStatus } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { cleanup, render, screen } from '@testing-library/react';
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
import { graphicItem } from '../../../shared/project-store/testing/graphics-fixture';
import {
  clipKind,
  fill,
  reframeSession,
} from '../../../shared/project-store/testing/reframe-fixture';
import { DEFAULT_TRACKING_SETTINGS } from '../services/tracking';
import { resetVisionStore } from '../store/vision-store';
import { AutoReframeControls } from './AutoReframeControls';

const clients: CoreClient[] = [];
afterEach(() => {
  // Unmount first: a mounted control would ask for the status again once it is reset.
  cleanup();
  for (const client of clients.splice(0)) client.close();
  resetVisionStore();
});

const AVAILABLE: VisionStatus = {
  available: true,
  model: 'yunet-2026may',
  runtime: 'ONNX Runtime 1.31.0',
  accelerator: 'CPU',
  reason: null,
};

function renderControls(clips: readonly ItemEntity[], status: VisionStatus = AVAILABLE) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi.spyOn(client, 'send').mockImplementation((command: Command) => {
    if (command.kind === 'getVisionStatus') {
      return Promise.resolve({ kind: 'getVisionStatus', status } as never);
    }
    if (command.kind === 'autoReframeItems') {
      return Promise.resolve({
        kind: 'autoReframeItems',
        appliedItemIds: [...command.itemIds],
        waitingAssetIds: [],
        skipped: [],
      } as never);
    }
    return Promise.resolve({ kind: command.kind } as never);
  });
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(reframeSession());
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <ProjectStoreProvider store={store} sync={sync}>
          <TooltipProvider>
            <AutoReframeControls clips={clips} />
          </TooltipProvider>
        </ProjectStoreProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return send;
}

const reframes = (send: ReturnType<typeof renderControls>) =>
  send.mock.calls
    .map(([command]) => command)
    .filter((command) => command.kind === 'autoReframeItems');

describe('automatic reframing in the inspector', () => {
  it('reframes a clip with the default settings', async () => {
    const user = userEvent.setup();
    const send = renderControls([graphicItem('filled', clipKind(fill()))]);
    await user.click(await screen.findByRole('button', { name: 'Recadrer automatiquement' }));
    expect(reframes(send)).toEqual([
      {
        kind: 'autoReframeItems',
        projectId: 'project-1',
        itemIds: ['filled'],
        settings: DEFAULT_TRACKING_SETTINGS,
      },
    ]);
  });

  it('applies a changed setting of a tracked clip at once', async () => {
    const user = userEvent.setup();
    const tracked = graphicItem(
      'filled',
      clipKind({
        kind: 'tracked',
        focus: { kind: 'static', value: { x: 0.3, y: 0.5 } },
        tracking: DEFAULT_TRACKING_SETTINGS,
      }),
    );
    const send = renderControls([tracked]);
    expect(await screen.findByRole('button', { name: 'Relancer le recadrage' })).toBeEnabled();
    await user.click(screen.getByRole('switch', { name: 'Cadrage de groupe' }));
    expect(reframes(send).map((command) => command.settings.groupFraming)).toEqual([false]);
    expect(
      screen.getAllByRole('slider').map((slider) => slider.getAttribute('aria-label')),
    ).toEqual(['Fluidité', 'Ligne des yeux', 'Zone morte']);
  });

  it('explains when faces cannot be detected', async () => {
    renderControls([graphicItem('filled', clipKind(fill()))], {
      ...AVAILABLE,
      available: false,
      runtime: null,
      accelerator: null,
      reason: 'no runtime',
    });
    expect(
      await screen.findByText(
        'La détection des visages n’est pas disponible sur cet ordinateur : recadrez à la main.',
      ),
    ).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Recadrer automatiquement' })).toBeNull();
  });
});

/**
 * @vitest-environment jsdom
 */
import { CoreClient, type Command, type Flicks } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { CoreClientProvider } from '../../../shared/core';
import { resetFontCatalog } from '../../../shared/fonts';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { SECOND } from '../../../shared/project-store/testing/graphics-fixture';
import { reframeSession } from '../../../shared/project-store/testing/reframe-fixture';
import { reframeModeStore, stopReframing } from '../../../shared/reframe';
import { clearSelection, selectItems, setPlayheadTime } from '../../timeline';
import { InspectorView } from './InspectorView';

const clients: CoreClient[] = [];
afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  clearSelection();
  resetFontCatalog();
  stopReframing();
  setPlayheadTime('sequence-1', 0 as Flicks);
});

function renderInspector() {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command: Command) =>
      Promise.resolve(
        (command.kind === 'listFonts'
          ? { kind: 'listFonts', families: [] }
          : { kind: command.kind }) as never,
      ),
    );
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(reframeSession());
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

const sent = (send: ReturnType<typeof renderInspector>, kind: Command['kind']) =>
  send.mock.calls.map(([command]) => command).filter((command) => command.kind === kind);

describe('the « Cadrage » section', () => {
  it('switches a clip to fill around the centre in one step', async () => {
    const user = userEvent.setup();
    const send = renderInspector();
    act(() => {
      selectItems(['wide']);
    });
    expect(screen.getByRole('radio', { name: 'Ajuster' })).toBeChecked();
    expect(screen.queryByText('Zone gardée')).not.toBeInTheDocument();
    await user.click(screen.getByRole('radio', { name: 'Remplir' }));
    expect(sent(send, 'updateItems')).toEqual([
      {
        kind: 'updateItems',
        projectId: 'project-1',
        updates: [
          {
            itemId: 'wide',
            patch: {
              kind: 'mediaClip',
              reframe: { kind: 'fill', focus: { kind: 'static', value: { x: 0.5, y: 0.5 } } },
            },
          },
        ],
      },
    ]);
  });

  it('keeps the size on screen when switching to the natural size', async () => {
    const user = userEvent.setup();
    const send = renderInspector();
    act(() => {
      selectItems(['wide']);
    });
    await user.click(screen.getByRole('radio', { name: 'Personnalisé' }));
    // A 1920 × 1080 picture fits a 1920 × 1080 frame at scale 1 and is 1920 design pixels wide:
    // the same size, so the scale stays 1.
    expect(sent(send, 'updateItems')[0]).toMatchObject({
      updates: [{ itemId: 'wide', patch: { reframe: { kind: 'custom' } } }],
      animations: [
        { itemId: 'wide', animation: { property: 'scale', value: { value: { x: 1, y: 1 } } } },
      ],
    });
  });

  it('edits the kept area of a filling clip and reframes it in the viewer', async () => {
    const user = userEvent.setup();
    const send = renderInspector();
    act(() => {
      setPlayheadTime('sequence-1', (5 * SECOND) as Flicks);
      selectItems(['filled']);
    });
    expect(screen.getByRole('radio', { name: 'Remplir' })).toBeChecked();
    const x = screen.getByRole('spinbutton', { name: 'Zone gardée X' });
    expect(x).toHaveAttribute('aria-valuenow', '50');
    x.focus();
    await user.keyboard('{ArrowUp}');
    expect(sent(send, 'setItemAnimations').at(-1)).toMatchObject({
      animations: [
        {
          itemId: 'filled',
          animation: { property: 'reframeFocus', value: { kind: 'static', value: { x: 0.51 } } },
        },
      ],
    });
    await user.click(screen.getByRole('button', { name: 'Recadrer dans le viewer' }));
    expect(reframeModeStore.getState().itemId).toBe('filled');
  });
});

/**
 * @vitest-environment jsdom
 */
import { CoreClient, type Command, type Flicks } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { clearPropertyPreview, propertyPreviewStore } from '../../../shared/animation';
import { CoreClientProvider } from '../../../shared/core';
import { resetFontCatalog } from '../../../shared/fonts';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { clearSelection, selectItems, setPlayheadTime } from '../../timeline';
import { SECOND, graphicsSession } from '../../../shared/project-store/testing/graphics-fixture';
import { InspectorView } from './InspectorView';

const clients: CoreClient[] = [];
afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  clearSelection();
  clearPropertyPreview();
  resetFontCatalog();
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
  sync.load(graphicsSession());
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

describe('the inspector of graphics', () => {
  it('shows the text, effects and transform sections of a selected text', () => {
    renderInspector();
    act(() => {
      selectItems(['plain']);
    });
    expect(screen.getByRole('button', { name: 'Texte' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Effets du texte' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Transformation' })).toBeInTheDocument();
    expect(screen.getByRole('spinbutton', { name: 'Taille' })).toHaveAttribute(
      'aria-valuenow',
      '96',
    );
  });

  it('commits one step per gesture and previews while dragging', async () => {
    const send = renderInspector();
    act(() => {
      selectItems(['plain']);
    });
    const size = screen.getByRole('spinbutton', { name: 'Taille' });
    size.focus();
    await userEvent.keyboard('{ArrowUp}');
    expect(sent(send, 'setItemAnimations')).toEqual([
      {
        kind: 'setItemAnimations',
        projectId: 'project-1',
        animations: [
          {
            itemId: 'plain',
            animation: { property: 'fontSize', value: { kind: 'static', value: 97 } },
          },
        ],
      },
    ]);
    // A drag previews every movement and sends once on release.
    fireEvent.pointerDown(size, { clientX: 0, button: 0, pointerId: 1 });
    fireEvent.pointerMove(size, { clientX: 20, pointerId: 1 });
    expect(propertyPreviewStore.getState().animations['plain']).toBeDefined();
    fireEvent.pointerMove(size, { clientX: 40, pointerId: 1 });
    expect(sent(send, 'setItemAnimations')).toHaveLength(1);
    fireEvent.pointerUp(size, { clientX: 40, pointerId: 1 });
    expect(sent(send, 'setItemAnimations')).toHaveLength(2);
  });

  it('adds a keyframe at the playhead to a keyframed property and toggles keyframes', async () => {
    const send = renderInspector();
    act(() => {
      selectItems(['title']);
      setPlayheadTime('sequence-1', (2 * SECOND) as Flicks);
    });
    const opacity = screen.getByRole('spinbutton', { name: 'Opacité' });
    opacity.focus();
    await userEvent.keyboard('{ArrowDown}');
    const [change] = sent(send, 'setItemAnimations');
    expect(change).toMatchObject({
      animations: [
        {
          itemId: 'title',
          animation: {
            property: 'opacity',
            value: {
              kind: 'keyframed',
              keyframes: [
                { time: 0, value: 0 },
                { time: SECOND, value: 1 },
                { time: 2 * SECOND, value: 0.99 },
              ],
            },
          },
        },
      ],
    });
    await userEvent.click(screen.getByRole('button', { name: 'Image clé : Rotation' }));
    const [, toggled] = sent(send, 'setItemAnimations');
    expect(toggled).toMatchObject({
      animations: [
        {
          itemId: 'title',
          animation: {
            property: 'rotation',
            value: { kind: 'keyframed', keyframes: [{ time: 2 * SECOND, value: 0 }] },
          },
        },
      ],
    });
  });

  it('patches static style values of every selected text', async () => {
    const send = renderInspector();
    act(() => {
      selectItems(['title', 'plain']);
    });
    await userEvent.click(screen.getByRole('switch', { name: 'Contour' }));
    expect(sent(send, 'updateItems')).toEqual([
      {
        kind: 'updateItems',
        projectId: 'project-1',
        updates: [
          { itemId: 'title', patch: { kind: 'text', strokeEnabled: true } },
          { itemId: 'plain', patch: { kind: 'text', strokeEnabled: true } },
        ],
      },
    ]);
  });
});

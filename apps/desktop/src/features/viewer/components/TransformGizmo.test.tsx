/** @vitest-environment jsdom */
import { CoreClient, type Command, type Flicks } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { clearPropertyPreview, propertyPreviewStore } from '../../../shared/animation';
import { CoreClientProvider } from '../../../shared/core';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { ShortcutProvider, ShortcutRegistry } from '../../../shared/shortcuts';
import { graphicsSession } from '../../../shared/project-store/testing/graphics-fixture';
import { clearSelection, playheadStore, selectionStore, selectItems } from '../../timeline';
import { DEFAULT_VIEWER_PREFERENCES, viewerStore } from '../store/viewer-store';
import { fakeAudioFactory } from '../testing/fake-audio-engine';
import { fakeEngineFactory, type FakePreviewEngine } from '../testing/fake-preview-engine';
import { ViewerPanel } from './ViewerPanel';

const clients: CoreClient[] = [];

function renderViewer() {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command: Command) => Promise.resolve({ kind: command.kind } as never));
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(graphicsSession());
  const fake = fakeEngineFactory();
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <ShortcutProvider registry={new ShortcutRegistry('other')}>
          <ProjectStoreProvider store={store} sync={sync}>
            <TooltipProvider>
              <ViewerPanel createEngine={fake.factory} createAudio={fakeAudioFactory().factory} />
            </TooltipProvider>
          </ProjectStoreProvider>
        </ShortcutProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return { send, engines: fake.engines };
}

async function runningEngine(engines: FakePreviewEngine[]): Promise<FakePreviewEngine> {
  await waitFor(() => {
    expect(engines.at(-1)?.scene).not.toBeNull();
  });
  const engine = engines.at(-1);
  if (engine === undefined) throw new Error('no engine started');
  return engine;
}

/** The two texts drawn centred (`plain` on top), 400 × 120 frame pixels each. */
function drawLayout(engine: FakePreviewEngine) {
  const transform = {
    position: { x: 0.5, y: 0.5 },
    scale: { x: 1, y: 1 },
    rotation: 0,
    anchor: { x: 0.5, y: 0.5 },
    opacity: 1,
    blur: 0,
  };
  act(() => {
    engine.emitLayout({
      time: 0 as Flicks,
      frame: { width: 1920, height: 1080 },
      layers: [
        { itemId: 'title', transform, box: { width: 600, height: 200 } },
        { itemId: 'plain', transform, box: { width: 400, height: 120 } },
      ],
    });
  });
}

const gizmo = () => {
  const element = document.querySelector<HTMLElement>('[data-viewer-gizmo]');
  if (element === null) throw new Error('no gizmo');
  return element;
};

beforeEach(() => {
  window.localStorage.clear();
  // A fixed zoom (saved, as the viewer restores its preferences): in jsdom the stage measures
  // 0 px. Half size: one CSS pixel is two frame pixels.
  window.localStorage.setItem('se.ui.viewer.v1', JSON.stringify({ zoom: 0.5 }));
  viewerStore.setState(DEFAULT_VIEWER_PREFERENCES);
  playheadStore.setState({ times: {} });
});

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  clearSelection();
  clearPropertyPreview();
});

describe('the transform gizmo', () => {
  it('selects the topmost layer under a click and clears the selection elsewhere', async () => {
    const { engines } = renderViewer();
    drawLayout(await runningEngine(engines));
    fireEvent.pointerDown(gizmo(), { clientX: 480, clientY: 270, button: 0, pointerId: 1 });
    fireEvent.pointerUp(gizmo(), { clientX: 480, clientY: 270, pointerId: 1 });
    expect(selectionStore.getState().itemIds).toEqual(['plain']);
    expect(screen.getAllByLabelText('Poignée de transformation')).toHaveLength(10);
    fireEvent.pointerDown(gizmo(), { clientX: 10, clientY: 10, button: 0, pointerId: 1 });
    expect(selectionStore.getState().itemIds).toEqual([]);
  });

  it('previews a move while dragging and sends one change on release', async () => {
    const { engines, send } = renderViewer();
    drawLayout(await runningEngine(engines));
    act(() => {
      selectItems(['plain']);
    });
    fireEvent.pointerDown(gizmo(), { clientX: 480, clientY: 270, button: 0, pointerId: 1 });
    fireEvent.pointerMove(gizmo(), { clientX: 505, clientY: 270, pointerId: 1 });
    fireEvent.pointerMove(gizmo(), { clientX: 530, clientY: 270, pointerId: 1 });
    expect(propertyPreviewStore.getState().animations['plain']).toEqual([
      { property: 'position', value: { kind: 'static', value: { x: 0.5 + 100 / 1920, y: 0.5 } } },
    ]);
    expect(send.mock.calls.some(([command]) => command.kind === 'setItemAnimations')).toBe(false);
    fireEvent.pointerUp(gizmo(), { clientX: 530, clientY: 270, pointerId: 1 });
    const changes = send.mock.calls
      .map(([command]) => command)
      .filter((command) => command.kind === 'setItemAnimations');
    expect(changes).toEqual([
      {
        kind: 'setItemAnimations',
        projectId: 'project-1',
        animations: [
          {
            itemId: 'plain',
            animation: {
              property: 'position',
              value: { kind: 'static', value: { x: 0.5 + 100 / 1920, y: 0.5 } },
            },
          },
        ],
      },
    ]);
  });

  it('scales from a side handle in one step', async () => {
    const { engines, send } = renderViewer();
    drawLayout(await runningEngine(engines));
    act(() => {
      selectItems(['plain']);
    });
    const handle = document.querySelector('[data-gizmo-handle="scale-e"]');
    if (handle === null) throw new Error('no handle');
    // The right edge is 200 frame px (100 CSS px) right of the anchor: dragged 50 CSS px further.
    fireEvent.pointerDown(handle, { clientX: 580, clientY: 270, button: 0, pointerId: 1 });
    fireEvent.pointerMove(gizmo(), { clientX: 630, clientY: 270, pointerId: 1 });
    fireEvent.pointerUp(gizmo(), { clientX: 630, clientY: 270, pointerId: 1 });
    const [change] = send.mock.calls
      .map(([command]) => command)
      .filter((command) => command.kind === 'setItemAnimations');
    expect(change).toMatchObject({
      animations: [
        {
          itemId: 'plain',
          animation: { property: 'scale', value: { kind: 'static', value: { x: 1.5, y: 1 } } },
        },
      ],
    });
  });
});

/** @vitest-environment jsdom */
import { CoreClient, type Command, type Flicks } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
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
import { SECOND } from '../../../shared/project-store/testing/graphics-fixture';
import { reframeSession } from '../../../shared/project-store/testing/reframe-fixture';
import { reframeModeStore, startReframing, stopReframing } from '../../../shared/reframe';
import { ShortcutProvider, ShortcutRegistry } from '../../../shared/shortcuts';
import { clearSelection, playheadStore, setPlayheadTime } from '../../timeline';
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
  sync.load(reframeSession());
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

/** The clip `filled` drawn over the whole frame, showing the middle half of its picture. */
function drawFilledClip(engine: FakePreviewEngine) {
  act(() => {
    engine.emitLayout({
      time: (5 * SECOND) as Flicks,
      frame: { width: 1920, height: 1080 },
      layers: [
        {
          itemId: 'filled',
          transform: {
            position: { x: 0.5, y: 0.5 },
            scale: { x: 1, y: 1 },
            rotation: 0,
            anchor: { x: 0.5, y: 0.5 },
            opacity: 1,
            blur: 0,
          },
          box: { width: 1920, height: 1080 },
          crop: { x: 0.25, y: 0, width: 0.5, height: 1 },
        },
      ],
    });
  });
}

const overlay = () => {
  const element = document.querySelector<HTMLElement>('[data-viewer-reframe]');
  if (element === null) throw new Error('no reframing overlay');
  return element;
};

beforeEach(() => {
  window.localStorage.clear();
  // Half size: one CSS pixel is two frame pixels (jsdom measures 0 px).
  window.localStorage.setItem('se.ui.viewer.v1', JSON.stringify({ zoom: 0.5 }));
  viewerStore.setState(DEFAULT_VIEWER_PREFERENCES);
  playheadStore.setState({ times: {} });
});

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  clearSelection();
  clearPropertyPreview();
  stopReframing();
});

describe('viewer guides', () => {
  it('draws the safe areas chosen for the sequence, never for the others', async () => {
    const user = userEvent.setup();
    const { engines } = renderViewer();
    await runningEngine(engines);
    expect(document.querySelector('[data-frame-guides]')).toBeNull();
    await user.click(screen.getByRole('button', { name: 'Repères et zones de sécurité' }));
    await user.click(screen.getByRole('menuitemradio', { name: 'TikTok' }));
    expect(document.querySelectorAll('[data-safe-zone]')).toHaveLength(4);
    // Measured for 9:16: indicative on this 16:9 sequence.
    expect(screen.getByText(/Zones TikTok prévues pour un autre format/)).toBeInTheDocument();
    expect(viewerStore.getState().guides['sequence-1']?.safeArea).toBe('tiktok');
    expect(viewerStore.getState().guides['sequence-2']).toBeUndefined();
  });
});

describe('reframing in the viewer', () => {
  it('drags the picture under the frame to move the kept part, in one step', async () => {
    const { engines, send } = renderViewer();
    const engine = await runningEngine(engines);
    act(() => {
      setPlayheadTime('sequence-1', (5 * SECOND) as Flicks);
      startReframing('filled');
    });
    drawFilledClip(engine);
    expect(document.querySelector('[data-viewer-gizmo]')).toBeNull();
    fireEvent.pointerDown(overlay(), { clientX: 480, clientY: 270, button: 0, pointerId: 1 });
    // 100 CSS pixels right = 200 frame pixels of a 3840 px wide picture.
    fireEvent.pointerMove(overlay(), { clientX: 580, clientY: 270, pointerId: 1 });
    const focus = { x: 0.5 - 200 / 3840, y: 0.5 };
    expect(propertyPreviewStore.getState().animations['filled']).toEqual([
      { property: 'reframeFocus', value: { kind: 'static', value: focus } },
    ]);
    fireEvent.pointerUp(overlay(), { clientX: 580, clientY: 270, pointerId: 1 });
    const changes = send.mock.calls
      .map(([command]) => command)
      .filter((command) => command.kind === 'setItemAnimations');
    expect(changes).toEqual([
      {
        kind: 'setItemAnimations',
        projectId: 'project-1',
        animations: [
          {
            itemId: 'filled',
            animation: { property: 'reframeFocus', value: { kind: 'static', value: focus } },
          },
        ],
      },
    ]);
  });

  it('starts on a double click on a clip that fills the frame', async () => {
    const { engines } = renderViewer();
    const engine = await runningEngine(engines);
    drawFilledClip(engine);
    const gizmo = document.querySelector<HTMLElement>('[data-viewer-gizmo]');
    if (gizmo === null) throw new Error('no gizmo');
    fireEvent.doubleClick(gizmo, { clientX: 480, clientY: 270 });
    expect(reframeModeStore.getState().itemId).toBe('filled');
  });

  it('leaves the mode on Enter and when the clip no longer fills the frame', async () => {
    const { engines } = renderViewer();
    const engine = await runningEngine(engines);
    act(() => {
      startReframing('filled');
    });
    drawFilledClip(engine);
    fireEvent.keyDown(window, { key: 'Enter' });
    expect(reframeModeStore.getState().itemId).toBeNull();
    act(() => {
      startReframing('wide');
    });
    // `wide` fits the frame: nothing to reframe.
    await waitFor(() => {
      expect(reframeModeStore.getState().itemId).toBeNull();
    });
    expect(document.querySelector('[data-viewer-reframe]')).toBeNull();
  });
});

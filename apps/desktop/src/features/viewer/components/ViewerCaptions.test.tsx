/** @vitest-environment jsdom */
import { CoreClient, type Command, type Flicks } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import {
  PREVIEW_CAPTION_STYLE_ID,
  captionPreviewStore,
  clearCaptionPreview,
} from '../../../shared/caption-preview';
import { CoreClientProvider } from '../../../shared/core';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { ShortcutProvider, ShortcutRegistry } from '../../../shared/shortcuts';
import { captionsStore } from '../../captions';
import { captionSession } from '../../../shared/project-store/testing/caption-fixture';
import { clearSelection, playheadStore, selectionStore } from '../../timeline';
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
    .mockImplementation((command: Command) =>
      Promise.resolve({ kind: command.kind, styleIds: [] } as never),
    );
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(captionSession());
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

/** « Bonjour à tous » drawn with its bottom centre at (0.5, 0.9): frame pixels 560–1360 × 872–972. */
function drawCaption(engine: FakePreviewEngine) {
  act(() => {
    engine.emitLayout({
      time: 0 as Flicks,
      frame: { width: 1920, height: 1080 },
      layers: [
        {
          itemId: 'caption-1',
          transform: {
            position: { x: 0.5, y: 0.9 },
            scale: { x: 1, y: 1 },
            rotation: 0,
            anchor: { x: 0.5, y: 1 },
            opacity: 1,
            blur: 0,
          },
          box: { width: 800, height: 100 },
          crop: null,
        },
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
  // Half size (jsdom measures 0 px): one CSS pixel is two frame pixels.
  window.localStorage.setItem('se.ui.viewer.v1', JSON.stringify({ zoom: 0.5 }));
  viewerStore.setState(DEFAULT_VIEWER_PREFERENCES);
  playheadStore.setState({ times: {} });
  captionsStore.setState({ scope: 'track', preset: 'karaoke' });
});

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  clearSelection();
  clearCaptionPreview();
});

describe('captions in the viewer', () => {
  it('draws captions with the project caption styles', async () => {
    const { engines } = renderViewer();
    const engine = await runningEngine(engines);
    expect(engine.scene?.sequence.captionStyles?.map((style) => style.id)).toEqual([
      'style-1',
      'style-2',
    ]);
  });

  it('dragging a caption moves its style for the whole track, without transform handles', async () => {
    // The karaoke style sits at 0.86; Ctrl/⌘ turns snapping off.
    const { engines, send } = renderViewer();
    const engine = await runningEngine(engines);
    drawCaption(engine);
    fireEvent.pointerDown(gizmo(), { clientX: 480, clientY: 460, button: 0, pointerId: 1 });
    expect(selectionStore.getState().itemIds).toEqual(['caption-1']);
    expect(screen.queryAllByLabelText('Poignée de transformation')).toHaveLength(0);
    fireEvent.pointerMove(gizmo(), { clientX: 480, clientY: 410, pointerId: 1, ctrlKey: true });
    fireEvent.pointerMove(gizmo(), { clientX: 480, clientY: 360, pointerId: 1, ctrlKey: true });
    const preview = captionPreviewStore.getState().preview;
    expect(preview?.itemIds).toEqual(['caption-1', 'caption-2']);
    expect(preview?.style.position.y).toBeCloseTo(0.86 - 200 / 1080);
    await waitFor(() => {
      const items = engine.scene?.sequence.tracks.flatMap((track) => track.items) ?? [];
      const dragged = items.find((item) => item.id === 'caption-1');
      expect(dragged?.kind.kind === 'caption' && dragged.kind.styleId).toBe(
        PREVIEW_CAPTION_STYLE_ID,
      );
    });
    fireEvent.pointerUp(gizmo(), { clientX: 480, clientY: 360, pointerId: 1, ctrlKey: true });
    const update = send.mock.calls
      .map(([command]) => command)
      .find((command) => command.kind === 'updateCaptionStyle');
    expect(update).toMatchObject({ itemIds: ['caption-1', 'caption-2'] });
    expect(update?.kind === 'updateCaptionStyle' && update.patch.position?.y).toBeCloseTo(
      0.86 - 200 / 1080,
    );
  });
});

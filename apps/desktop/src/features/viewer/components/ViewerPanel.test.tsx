/** @vitest-environment jsdom */
import { CoreClient, type Flicks, type ProjectSession } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, render, screen, waitFor } from '@testing-library/react';
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
import { IDS, session } from '../../../shared/project-store/testing/session-fixture';
import { ShortcutProvider, ShortcutRegistry } from '../../../shared/shortcuts';
import { playheadStore } from '../../timeline';
import { DEFAULT_VIEWER_PREFERENCES, viewerStore } from '../store/viewer-store';
import { fakeEngineFactory, type FakePreviewEngine } from '../testing/fake-preview-engine';
import { ViewerPanel } from './ViewerPanel';

function renderViewer(snapshot: ProjectSession | null = session()) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  if (snapshot !== null) sync.load(snapshot);
  const fake = fakeEngineFactory();
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <ShortcutProvider registry={new ShortcutRegistry('other')}>
          <ProjectStoreProvider store={store} sync={sync}>
            <TooltipProvider>
              <ViewerPanel createEngine={fake.factory} />
            </TooltipProvider>
          </ProjectStoreProvider>
        </ShortcutProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return fake;
}

async function runningEngine(engines: FakePreviewEngine[]): Promise<FakePreviewEngine> {
  await waitFor(() => {
    expect(engines.at(-1)?.scene).not.toBeNull();
  });
  const engine = engines.at(-1);
  if (engine === undefined) throw new Error('no engine started');
  return engine;
}

beforeEach(() => {
  window.localStorage.clear();
  viewerStore.setState(DEFAULT_VIEWER_PREFERENCES);
  playheadStore.setState({ times: {} });
});

afterEach(() => {
  vi.useRealTimers();
});

describe('ViewerPanel', () => {
  it('shows an empty state without a sequence', () => {
    const { engines } = renderViewer(null);
    expect(screen.getByText('Aucune séquence ouverte')).toBeInTheDocument();
    expect(engines).toHaveLength(0);
  });

  it('draws the active sequence at the playhead and follows it', async () => {
    const { engines } = renderViewer();
    const engine = await runningEngine(engines);
    expect(engine.scene?.sequence.tracks).toHaveLength(2);
    expect(engine.time).toBe(0);
    act(() => {
      playheadStore.setState({ times: { [IDS.sequence]: 705_600_000 as Flicks } });
    });
    expect(engine.time).toBe(705_600_000);
    expect(screen.getByRole('timer', { name: 'Position de la tête de lecture' })).toHaveTextContent(
      '00:00:01:00',
    );
    expect(screen.getByText('1920 × 1080 · 25 i/s')).toBeInTheDocument();
  });

  it('sizes the picture by the zoom and switches proxy and checkerboard', async () => {
    const user = userEvent.setup();
    const { engines } = renderViewer();
    const engine = await runningEngine(engines);
    await user.click(screen.getByRole('combobox', { name: 'Zoom du viewer' }));
    await user.click(screen.getByRole('option', { name: '50 %' }));
    const picture = screen.getByRole('img', { name: 'Image de la séquence' });
    expect(picture).toHaveStyle({ width: '960px', height: '540px' });
    expect(engine.size).toEqual([960, 540]);
    await user.click(screen.getByRole('button', { name: 'Lire les proxys' }));
    expect(engine.proxy).toBe(true);
    await user.click(screen.getByRole('button', { name: 'Damier de transparence' }));
    expect(engine.background).toBe('transparent');
    expect(JSON.parse(window.localStorage.getItem('se.ui.viewer.v1') ?? '{}')).toEqual({
      zoom: 0.5,
      proxy: true,
      checkerboard: true,
    });
  });

  it('shows a spinner while decoding takes time, and media problems', async () => {
    const { engines } = renderViewer();
    const engine = await runningEngine(engines);
    vi.useFakeTimers();
    act(() => {
      engine.emit({ status: 'loading', issues: [], error: null });
    });
    expect(screen.queryByRole('status', { name: 'Décodage de l’image…' })).toBeNull();
    act(() => {
      vi.advanceTimersByTime(200);
    });
    expect(screen.getByRole('status', { name: 'Décodage de l’image…' })).toBeInTheDocument();
    act(() => {
      engine.emit({ status: 'ready', issues: [{ assetId: 'a', kind: 'missing' }], error: null });
    });
    expect(screen.queryByRole('status', { name: 'Décodage de l’image…' })).toBeNull();
    expect(screen.getByText('1 média hors ligne')).toBeInTheDocument();
  });

  it('explains when the engine cannot start', async () => {
    const client = CoreClient.connect(() => new Promise(() => undefined));
    const store = createProjectStore();
    const sync = connectProjectSync(client, store);
    sync.load(session());
    render(
      <I18nextProvider i18n={createI18n()}>
        <CoreClientProvider client={client}>
          <ShortcutProvider registry={new ShortcutRegistry('other')}>
            <ProjectStoreProvider store={store} sync={sync}>
              <TooltipProvider>
                <ViewerPanel />
              </TooltipProvider>
            </ProjectStoreProvider>
          </ShortcutProvider>
        </CoreClientProvider>
      </I18nextProvider>,
    );
    expect(await screen.findByText('Aperçu indisponible')).toBeInTheDocument();
    expect(
      screen.getByText('Cette machine ne permet pas de décoder la vidéo (WebCodecs absent).'),
    ).toBeInTheDocument();
  });
});

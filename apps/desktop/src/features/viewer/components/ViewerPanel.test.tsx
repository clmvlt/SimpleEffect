/** @vitest-environment jsdom */
import { CoreClient, type Flicks, type ItemEntity, type ProjectSession } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, fireEvent, render, screen, waitFor, within } from '@testing-library/react';
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
import { captionItem, IDS, session } from '../../../shared/project-store/testing/session-fixture';
import { ShortcutProvider, ShortcutRegistry } from '../../../shared/shortcuts';
import { playheadStore, playheadTimeOf } from '../../timeline';
import { DEFAULT_VIEWER_PREFERENCES, viewerStore } from '../store/viewer-store';
import { fakeAudioFactory, type FakeAudioEngine } from '../testing/fake-audio-engine';
import { fakeEngineFactory, type FakePreviewEngine } from '../testing/fake-preview-engine';
import { ViewerPanel } from './ViewerPanel';

const SECOND = 705_600_000;

/** The fixture session with its item lasting ten seconds (room to play). */
function tenSecondSession(): ProjectSession {
  const base = session();
  const item: ItemEntity = {
    ...captionItem(IDS.item, IDS.v1, 0),
    timelineRange: { start: 0 as Flicks, duration: (10 * SECOND) as Flicks },
  };
  return { ...base, snapshot: { ...base.snapshot, items: [item] } };
}

function renderViewer(snapshot: ProjectSession | null = session()) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  if (snapshot !== null) sync.load(snapshot);
  const fake = fakeEngineFactory();
  const audio = fakeAudioFactory();
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <ShortcutProvider registry={new ShortcutRegistry('other')}>
          <ProjectStoreProvider store={store} sync={sync}>
            <TooltipProvider>
              <ViewerPanel createEngine={fake.factory} createAudio={audio.factory} />
            </TooltipProvider>
          </ProjectStoreProvider>
        </ShortcutProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return { ...fake, audios: audio.engines };
}

async function runningAudio(audios: FakeAudioEngine[]): Promise<FakeAudioEngine> {
  await waitFor(() => {
    expect(audios.at(-1)?.scene).not.toBeNull();
  });
  const audio = audios.at(-1);
  if (audio === undefined) throw new Error('no audio engine started');
  return audio;
}

/** Waits for the next display refresh (playback ticks run on animation frames). */
async function nextFrame(): Promise<void> {
  await act(
    () =>
      new Promise<void>((resolve) =>
        requestAnimationFrame(() => {
          resolve();
        }),
      ),
  );
}

const playheadTime = () => playheadTimeOf(playheadStore.getState(), IDS.sequence);

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
    expect(screen.getByRole('timer', { name: 'Durée de la séquence' })).toHaveTextContent(
      '00:00:00:00',
    );
  });

  it('sizes the picture by the zoom and switches proxy and checkerboard', async () => {
    const user = userEvent.setup();
    const { engines } = renderViewer();
    const engine = await runningEngine(engines);
    await user.click(screen.getByRole('combobox', { name: 'Zoom du viewer' }));
    await user.click(screen.getByRole('option', { name: '50 %' }));
    const picture = screen.getByRole('img', { name: 'Image de la séquence' });
    // The picture fills a wrapper of the displayed size (the transform handles sit over it).
    expect(picture.parentElement).toHaveStyle({ width: '960px', height: '540px' });
    expect(engine.size).toEqual([960, 540]);
    await user.click(screen.getByRole('button', { name: 'Lire les proxys' }));
    expect(engine.proxy).toBe(true);
    await user.click(screen.getByRole('button', { name: 'Damier de transparence' }));
    expect(engine.background).toBe('transparent');
    expect(JSON.parse(window.localStorage.getItem('se.ui.viewer.v1') ?? '{}')).toEqual({
      ...DEFAULT_VIEWER_PREFERENCES,
      zoom: 0.5,
      proxy: true,
      checkerboard: true,
    });
  });

  it('plays with Space on the audio clock and pauses with the transport bar', async () => {
    const user = userEvent.setup();
    const { engines, audios } = renderViewer(tenSecondSession());
    const engine = await runningEngine(engines);
    const audio = await runningAudio(audios);
    expect(screen.getByRole('timer', { name: 'Durée de la séquence' })).toHaveTextContent(
      '00:00:10:00',
    );
    await user.click(screen.getByRole('img', { name: 'Image de la séquence' }));
    await user.keyboard(' ');
    expect(audio.transport?.rate).toBe(1);
    expect(engine.playback).toEqual({ rate: 1, frameRate: { numerator: 25, denominator: 1 } });
    audio.now = 2.01;
    await nextFrame();
    expect(playheadTime()).toBe(2 * SECOND);
    expect(engine.time).toBe(2 * SECOND);
    await user.click(screen.getByRole('button', { name: 'Pause' }));
    expect(audio.transport).toBeNull();
    expect(engine.playback).toBeNull();
    expect(screen.getByRole('button', { name: 'Lecture' })).toBeInTheDocument();
  });

  it('shuttles with JKL, steps with K held, toggles the loop with Ctrl+L', async () => {
    const user = userEvent.setup();
    const { audios } = renderViewer(tenSecondSession());
    const audio = await runningAudio(audios);
    await user.click(screen.getByRole('img', { name: 'Image de la séquence' }));
    await user.keyboard('ll');
    expect(audio.transport?.rate).toBe(2);
    await user.keyboard('k');
    expect(audio.transport).toBeNull();
    await user.keyboard('{k>}lll{/k}');
    expect(playheadTime()).toBe(3 * (SECOND / 25));
    await user.keyboard('{k>}j{/k}');
    expect(playheadTime()).toBe(2 * (SECOND / 25));
    await user.keyboard('j');
    expect(audio.transport?.rate).toBe(-1);
    await user.keyboard('{Control>}l{/Control}');
    expect(viewerStore.getState().loop).toBe(true);
    expect(screen.getByRole('button', { name: /Lecture en boucle/ })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
  });

  it('steps, stops and goes to the edges from the transport bar, scrubbing the sound', async () => {
    const user = userEvent.setup();
    const { audios } = renderViewer(tenSecondSession());
    const audio = await runningAudio(audios);
    const transport = screen.getByRole('toolbar', { name: 'Transport' });
    await user.click(within(transport).getByRole('button', { name: 'Image suivante' }));
    expect(playheadTime()).toBe(SECOND / 25);
    expect(audio.scrubs).toEqual([SECOND / 25]);
    await user.click(within(transport).getByRole('button', { name: 'Aller à la fin' }));
    expect(playheadTime()).toBe(10 * SECOND);
    await user.click(within(transport).getByRole('button', { name: 'Aller au début' }));
    expect(playheadTime()).toBe(0);
    await user.click(screen.getByRole('button', { name: /Son pendant le déplacement/ }));
    expect(viewerStore.getState().scrubAudio).toBe(false);
    act(() => {
      playheadStore.setState({ times: { [IDS.sequence]: (4 * SECOND) as Flicks } });
    });
    expect(audio.scrubs).toHaveLength(3);
  });

  it('shows the master levels and, when asked, the performance HUD', async () => {
    const user = userEvent.setup();
    const { engines } = renderViewer(tenSecondSession());
    await runningEngine(engines);
    const meter = screen.getByRole('group', { name: 'Niveau de sortie' });
    await waitFor(() => {
      expect(within(meter).getByRole('meter', { name: 'Gauche' })).toHaveAttribute(
        'aria-valuenow',
        '-12',
      );
    });
    await user.click(screen.getByRole('button', { name: 'Statistiques de lecture' }));
    const hud = screen.getByRole('group', { name: 'Statistiques de lecture' });
    expect(hud).toHaveTextContent('1920 × 1080 · 25 i/s');
    expect(hud).toHaveTextContent('2 images perdues');
    expect(hud).toHaveTextContent('Cache d’images : 24 Mo (3 images)');
    fireEvent.click(screen.getByRole('button', { name: 'Statistiques de lecture' }));
    expect(screen.queryByRole('group', { name: 'Statistiques de lecture' })).toBeNull();
  });

  it('shows a spinner while decoding takes time, and media problems', async () => {
    const { engines } = renderViewer();
    const engine = await runningEngine(engines);
    vi.useFakeTimers();
    act(() => {
      engine.emit({ status: 'loading', issues: [], missingFonts: [], error: null });
    });
    expect(screen.queryByRole('status', { name: 'Décodage de l’image…' })).toBeNull();
    act(() => {
      vi.advanceTimersByTime(200);
    });
    expect(screen.getByRole('status', { name: 'Décodage de l’image…' })).toBeInTheDocument();
    act(() => {
      engine.emit({
        status: 'ready',
        issues: [{ assetId: 'a', kind: 'missing' }],
        missingFonts: [],
        error: null,
      });
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

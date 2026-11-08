/**
 * @vitest-environment jsdom
 */
import {
  CoreClient,
  type Command,
  type CutSummary,
  type ProjectSession,
  type SmartCutSettings,
  type Transcript,
} from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { SimpleEffectBridge } from '../../../../electron/shared/bridge-api';
import { createI18n } from '../../../i18n';
import { CoreClientProvider } from '../../../shared/core';
import { DesktopBridgeProvider } from '../../../shared/desktop';
import { onPlayRequest } from '../../../shared/playback';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { transform } from '../../../shared/project-store/testing/graphics-fixture';
import { session } from '../../../shared/project-store/testing/session-fixture';
import { ShortcutProvider, ShortcutRegistry } from '../../../shared/shortcuts';
import { cutPreviewStore, playheadStore, playheadTimeOf, useInOutPoints } from '../../timeline';
import { seconds, testAsset, testItem, testSequence, testTrack } from '../testing/fixtures';
import { smartCutSettingsStore } from '../store/smart-cut-settings-store';
import { clearTranscripts } from '../store/transcripts-store';
import { TranscriptView } from './TranscriptView';

const idleBridge = {
  getCoreConnection: () => new Promise(() => undefined),
} as unknown as SimpleEffectBridge;

const TRANSCRIPT: Transcript = {
  language: 'fr',
  model: 'base',
  words: [
    { text: 'Bonjour', start: seconds(0), end: seconds(0.4), confidence: 0.95, segment: 0 },
    { text: 'à', start: seconds(0.4), end: seconds(0.5), confidence: 0.95, segment: 0 },
    { text: 'tous.', start: seconds(0.5), end: seconds(0.9), confidence: 0.95, segment: 0 },
    { text: 'Abonnez-vous', start: seconds(2), end: seconds(2.8), confidence: 0.3, segment: 1 },
  ],
  segments: [
    { start: seconds(0), end: seconds(0.9) },
    { start: seconds(2), end: seconds(2.8) },
  ],
};

/** A sequence whose sound track plays the interview from 10 s, and a second media not transcribed. */
function transcriptSession(): ProjectSession {
  const base = session();
  const interview = {
    ...testAsset('asset-1', '/rushes/interview.mp4'),
    analysis: {
      audio: null,
      scenes: null,
      transcript: { language: 'fr', model: 'base', wordCount: 4, transcribedAt: 7 },
    },
  };
  const voice = testAsset('asset-2', '/rushes/voix.wav', 'audio');
  const clip = (id: string, assetId: string, start: number) =>
    testItem(id, 'a1', start, 5, {
      kind: {
        kind: 'mediaClip',
        assetId,
        sourceStart: seconds(0),
        speed: 1,
        transform: transform(),
        volumeDb: { kind: 'static', value: 0 },
        fadeIn: seconds(0),
        fadeOut: seconds(0),
        fadeInShape: 'linear',
        fadeOutShape: 'linear',
        reframe: { kind: 'fit' },
      },
    });
  return {
    ...base,
    snapshot: {
      ...base.snapshot,
      sequences: [testSequence(['a1'])],
      tracks: [testTrack('a1', 'audio', ['clip-1', 'clip-2'])],
      items: [clip('clip-1', 'asset-1', 10), clip('clip-2', 'asset-2', 20)],
      markers: [],
      assets: [interview, voice],
    },
  };
}

/**
 * The interview cut: « à tous. » (source 0.4–0.9 s) removed between two touching clips of it on
 * A1, picture linked on V1.
 */
function cutSession(): ProjectSession {
  const base = transcriptSession();
  const interview = base.snapshot.assets[0];
  const clip = (id: string, trackId: string, start: number, length: number, source: number) =>
    testItem(id, trackId, start, length, {
      linkGroup: start < 10.4 ? 'group-1' : 'group-2',
      kind: {
        kind: 'mediaClip',
        assetId: 'asset-1',
        sourceStart: seconds(source),
        speed: 1,
        transform: transform(),
        volumeDb: { kind: 'static', value: 0 },
        fadeIn: seconds(0),
        fadeOut: seconds(0),
        fadeInShape: 'linear',
        fadeOutShape: 'linear',
        reframe: { kind: 'fit' },
      },
    });
  return {
    ...base,
    snapshot: {
      ...base.snapshot,
      sequences: [testSequence(['v1', 'a1'])],
      tracks: [
        testTrack('v1', 'video', ['picture-1', 'picture-2']),
        testTrack('a1', 'audio', ['sound-1', 'sound-2']),
      ],
      items: [
        clip('picture-1', 'v1', 10, 0.4, 0),
        clip('picture-2', 'v1', 10.4, 2, 0.9),
        clip('sound-1', 'a1', 10, 0.4, 0),
        clip('sound-2', 'a1', 10.4, 2, 0.9),
      ],
      assets: interview === undefined ? [] : [interview],
    },
  };
}

const SETTINGS: SmartCutSettings = {
  fillers: [{ language: 'fr', words: ['euh'] }],
  fillerMaxDuration: seconds(1.5),
  silence: { thresholdDbfs: -40, minSilence: seconds(0.5), padding: seconds(0.12) },
};

function summary(cuts: [number, number][]): CutSummary {
  return {
    cuts: cuts.map(([start, end]) => ({ start: seconds(start), duration: seconds(end - start) })),
    removedDuration: seconds(cuts.reduce((total, [start, end]) => total + end - start, 0)),
    wordCount: 0,
    trackIds: ['a1'],
    untranscribedAssets: [],
    unanalysedAssets: [],
  };
}

const FILLERS = [
  { text: 'Euh', start: seconds(10.4), end: seconds(10.6), itemId: 'clip-1', assetId: 'asset-1' },
  { text: 'euh…', start: seconds(12), end: seconds(12.3), itemId: 'clip-1', assetId: 'asset-1' },
];

/** The core's answer to `command` in these tests. */
function answerTo(command: Command): unknown {
  switch (command.kind) {
    case 'getTranscript':
      return { kind: command.kind, transcript: TRANSCRIPT };
    case 'transcribeAssets':
      return { kind: command.kind, jobIds: ['job-1'] };
    case 'getSmartCutSettings':
      return { kind: command.kind, ...SETTINGS };
    case 'removeWords':
    case 'removeSilences':
      return { kind: command.kind, ...summary([[10.4, 10.9]]) };
    case 'previewSilenceCuts':
      return {
        kind: command.kind,
        ...summary([
          [11, 12],
          [13, 13.5],
        ]),
      };
    case 'findFillers':
      return { kind: command.kind, occurrences: FILLERS, untranscribedAssets: [] };
    default:
      return { kind: command.kind };
  }
}

/** Shows the timeline's in point (read through the timeline's public hook). */
function InOutProbe() {
  const { inPoint } = useInOutPoints('sequence-1');
  return <output aria-label="in">{inPoint === null ? '' : String(inPoint)}</output>;
}

const clients: CoreClient[] = [];

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  clearTranscripts();
  smartCutSettingsStore.setState({ settings: null });
});

function renderPanel(project: ProjectSession = transcriptSession()) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command: Command) => Promise.resolve(answerTo(command) as never));
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(project);
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <DesktopBridgeProvider bridge={idleBridge}>
          <ProjectStoreProvider store={store} sync={sync}>
            <ShortcutProvider registry={new ShortcutRegistry('other')}>
              <TooltipProvider>
                <TranscriptView />
                <InOutProbe />
              </TooltipProvider>
            </ShortcutProvider>
          </ProjectStoreProvider>
        </DesktopBridgeProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return { send };
}

describe('TranscriptView', () => {
  it('shows the sequence’s words in sequence time and moves the playhead to a clicked word', async () => {
    const { send } = renderPanel();
    const word = await screen.findByText('tous.');
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({ kind: 'getTranscript', assetId: 'asset-1' }),
    );
    expect(screen.getByText('0:10')).toBeInTheDocument();
    expect(screen.getByText('Abonnez-vous')).toHaveAttribute('data-unsure');
    fireEvent.pointerDown(word, { button: 0 });
    fireEvent.pointerUp(word, { button: 0 });
    expect(playheadTimeOf(playheadStore.getState(), 'sequence-1')).toBe(seconds(10.5));
  });

  it('marks dragged words as in and out', async () => {
    renderPanel();
    const first = await screen.findByText('Bonjour');
    const last = screen.getByText('tous.');
    fireEvent.pointerDown(first, { button: 0 });
    fireEvent.pointerMove(last);
    fireEvent.pointerUp(last);
    expect(screen.getByLabelText('in')).toHaveTextContent(String(seconds(10)));
  });

  it('searches with accents ignored and counts the matches', async () => {
    renderPanel();
    await screen.findByText('Bonjour');
    await userEvent.type(screen.getByRole('searchbox'), 'ABONN');
    expect(screen.getByText('Abonnez-vous')).toHaveAttribute('data-match');
    expect(screen.getByText('1/1')).toBeInTheDocument();
  });

  it('transcribes media without transcript', async () => {
    const { send } = renderPanel();
    await screen.findByText('voix.wav');
    await userEvent.click(screen.getByRole('button', { name: 'Transcrire' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'transcribeAssets',
      projectId: expect.any(String) as string,
      assetIds: ['asset-2'],
    });
  });

  it('saves the project’s language and vocabulary', async () => {
    const { send } = renderPanel();
    await screen.findByText('Bonjour');
    await userEvent.click(screen.getByRole('button', { name: 'Langue et vocabulaire' }));
    await userEvent.type(screen.getByRole('textbox', { name: 'Vocabulaire' }), 'SimpleEffect');
    await act(async () => {
      await userEvent.click(screen.getByRole('button', { name: 'Enregistrer' }));
    });
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: 'setTranscriptionSettings',
        settings: { language: 'fr', vocabulary: 'SimpleEffect' },
      }),
    );
  });

  it('cuts the selected words with Suppr', async () => {
    const { send } = renderPanel();
    const first = await screen.findByText('à');
    const last = screen.getByText('tous.');
    fireEvent.pointerDown(first, { button: 0 });
    fireEvent.pointerMove(last);
    fireEvent.pointerUp(last);
    await userEvent.keyboard('{Delete}');
    expect(send).toHaveBeenCalledWith({
      kind: 'removeWords',
      projectId: expect.any(String) as string,
      sequenceId: 'sequence-1',
      ranges: [{ start: seconds(10.4), duration: seconds(0.5) }],
      purpose: 'words',
      allTracks: false,
    });
  });

  it('shows cut passages struck through and restores one with a ripple trim', async () => {
    const { send } = renderPanel(cutSession());
    await screen.findByText('Bonjour');
    expect(screen.queryByText('à tous.')).not.toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Afficher les passages coupés' }));
    expect(screen.getByText('à tous.')).toHaveClass('line-through');
    await userEvent.click(screen.getByRole('button', { name: 'Restaurer ce passage' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'trimItems',
      projectId: expect.any(String) as string,
      trims: [
        { itemId: 'sound-1', edge: 'end', to: seconds(10.9) },
        { itemId: 'picture-1', edge: 'end', to: seconds(10.9) },
      ],
      ripple: true,
    });
  });

  it('previews silences on the timeline before removing them', async () => {
    const { send } = renderPanel();
    await userEvent.click(await screen.findByRole('radio', { name: 'Silences' }));
    expect(await screen.findByText('2 coupes · 1,5 s gagnées')).toBeInTheDocument();
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: 'previewSilenceCuts',
        range: null,
        params: SETTINGS.silence,
        trackIds: null,
        allTracks: false,
      }),
    );
    expect(cutPreviewStore.getState().ranges).toHaveLength(2);
    await userEvent.click(screen.getByRole('button', { name: 'Supprimer les silences (2)' }));
    expect(send).toHaveBeenCalledWith(expect.objectContaining({ kind: 'removeSilences' }));
    await userEvent.click(screen.getByRole('radio', { name: 'Texte' }));
    expect(cutPreviewStore.getState().ranges).toHaveLength(0);
  });

  it('lists fillers, plays one and removes the ticked ones', async () => {
    const { send } = renderPanel();
    const heard = vi.fn();
    const stop = onPlayRequest(heard);
    await userEvent.click(await screen.findByRole('radio', { name: 'Hésitations' }));
    expect(await screen.findByText('2 hésitations trouvées')).toBeInTheDocument();
    await userEvent.click(screen.getByRole('button', { name: 'Écouter « Euh »' }));
    expect(heard).toHaveBeenCalledWith({
      sequenceId: 'sequence-1',
      start: seconds(10),
      end: seconds(11),
    });
    await userEvent.click(screen.getByRole('checkbox', { name: 'Supprimer « euh… » à 0:12' }));
    await userEvent.click(screen.getByRole('button', { name: 'Supprimer la sélection (1)' }));
    await waitFor(() => {
      expect(send).toHaveBeenCalledWith(
        expect.objectContaining({
          kind: 'removeWords',
          ranges: [{ start: seconds(10.4), duration: seconds(0.2) }],
          purpose: 'fillers',
        }),
      );
    });
    stop();
  });
});

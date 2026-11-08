/**
 * @vitest-environment jsdom
 */
import { CoreClient, type Command, type ProjectSession, type Transcript } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { SimpleEffectBridge } from '../../../../electron/shared/bridge-api';
import { createI18n } from '../../../i18n';
import { CoreClientProvider } from '../../../shared/core';
import { DesktopBridgeProvider } from '../../../shared/desktop';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { transform } from '../../../shared/project-store/testing/graphics-fixture';
import { session } from '../../../shared/project-store/testing/session-fixture';
import { playheadStore, playheadTimeOf, useInOutPoints } from '../../timeline';
import { seconds, testAsset, testItem, testSequence, testTrack } from '../testing/fixtures';
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

/** Shows the timeline's in point (read through the timeline's public hook). */
function InOutProbe() {
  const { inPoint } = useInOutPoints('sequence-1');
  return <output aria-label="in">{inPoint === null ? '' : String(inPoint)}</output>;
}

const clients: CoreClient[] = [];

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  clearTranscripts();
});

function renderPanel() {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi.spyOn(client, 'send').mockImplementation((command: Command) => {
    const answer =
      command.kind === 'getTranscript'
        ? { kind: command.kind, transcript: TRANSCRIPT }
        : command.kind === 'transcribeAssets'
          ? { kind: command.kind, jobIds: ['job-1'] }
          : { kind: command.kind };
    return Promise.resolve(answer as never);
  });
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(transcriptSession());
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <DesktopBridgeProvider bridge={idleBridge}>
          <ProjectStoreProvider store={store} sync={sync}>
            <TooltipProvider>
              <TranscriptView />
              <InOutProbe />
            </TooltipProvider>
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
});

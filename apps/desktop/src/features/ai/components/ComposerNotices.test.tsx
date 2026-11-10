/**
 * @vitest-environment jsdom
 */
import {
  CoreClient,
  type AiStatus,
  type Command,
  type Flicks,
  type ItemEntity,
  type ProjectSession,
  type Task,
  type TranscriptionStatus,
} from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { SimpleEffectBridge } from '../../../../electron/shared/bridge-api';
import { createI18n } from '../../../i18n';
import { resetAiTasks, setProjectTasks } from '../../../shared/ai-tasks';
import { task as taskOf } from '../../../shared/ai-tasks/testing/task-fixture';
import { CoreClientProvider } from '../../../shared/core';
import { DesktopBridgeProvider } from '../../../shared/desktop';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { transform } from '../../../shared/project-store/testing/graphics-fixture';
import {
  IDS,
  captionKind,
  session,
  videoTrack,
} from '../../../shared/project-store/testing/session-fixture';
import { ShortcutProvider, ShortcutRegistry } from '../../../shared/shortcuts';
import { clearSelection, selectItems } from '../../timeline';
import { setAiStatus } from '../store/ai-status-store';
import { resetComposer } from '../store/composer-store';
import { TaskComposer } from './TaskComposer';

const SECOND = 705_600_000;

const READY: AiStatus = {
  provider: {
    id: 'claude-code',
    name: 'Claude Code',
    command: 'claude',
    loginCommand: '/login',
    helpUrl: 'https://code.claude.com/docs/en/setup',
  },
  installation: { kind: 'installed', path: '/bin/claude', version: '2.1.317' },
  auth: { kind: 'loggedIn', accountKind: 'max' },
  capabilities: { resumeSessions: true, imageInput: true, models: [], effortLevels: [] },
};

const idleBridge: SimpleEffectBridge = {
  getCoreConnection: () => new Promise(() => undefined),
  chooseFolder: () => Promise.resolve(null),
  chooseFiles: () => Promise.resolve([]),
  chooseFile: () => Promise.resolve(null),
  chooseSaveFile: () => Promise.resolve(null),
  openExternal: () => Promise.resolve(),
  showInFolder: () => Promise.resolve(),
  getPathForFile: () => '',
  showNotification: () => Promise.resolve(),
  resolveFilePath: ({ folder, fileName }) =>
    Promise.resolve({ path: `${folder}/${fileName}`, exists: false }),
  setExportActivity: () => Promise.resolve(),
  onNotificationClick: () => () => undefined,
};

/** A one-minute text on V2, so a one-frame selection on V1 is a small part of the sequence. */
const LONG_TEXT: ItemEntity = {
  trackId: IDS.v2,
  id: 'item-long',
  revision: 1,
  origin: { kind: 'user' },
  timelineRange: { start: 0 as Flicks, duration: (60 * SECOND) as Flicks },
  linkGroup: null,
  kind: captionKind('Générique'),
};

function longSession(): ProjectSession {
  const base = session();
  return {
    ...base,
    snapshot: {
      ...base.snapshot,
      tracks: [videoTrack(IDS.v1, [IDS.item]), videoTrack(IDS.v2, [LONG_TEXT.id])],
      items: [...base.snapshot.items, LONG_TEXT],
    },
  };
}

/** {@link longSession} plus an audio track playing `voix.mp4`, never transcribed. */
function speechSession(): ProjectSession {
  const base = longSession();
  const sequence = base.snapshot.sequences[0];
  if (sequence === undefined) throw new Error('the fixture has a sequence');
  const voice: ItemEntity = {
    trackId: 'track-a1',
    id: 'clip-voice',
    revision: 1,
    origin: { kind: 'user' },
    timelineRange: { start: 0 as Flicks, duration: (10 * SECOND) as Flicks },
    linkGroup: null,
    kind: {
      kind: 'mediaClip',
      assetId: 'voice',
      sourceStart: 0 as Flicks,
      speed: 1,
      transform: transform(),
      volumeDb: { kind: 'static', value: 0 },
      fadeIn: 0 as Flicks,
      fadeOut: 0 as Flicks,
      fadeInShape: 'linear',
      fadeOutShape: 'linear',
      reframe: { kind: 'fit' },
    },
  };
  return {
    ...base,
    snapshot: {
      ...base.snapshot,
      sequences: [{ ...sequence, trackIds: [...sequence.trackIds, 'track-a1'] }],
      tracks: [
        ...base.snapshot.tracks,
        { ...videoTrack('track-a1', [voice.id]), kind: 'audio', name: 'A1' },
      ],
      items: [...base.snapshot.items, voice],
      assets: [
        {
          id: 'voice',
          originalPath: '/rushes/voix.mp4',
          relativeHint: null,
          kind: 'video',
          mediaInfo: null,
          fingerprint: null,
          derived: { mezzanine: null, proxy: null },
          analysis: { audio: null, scenes: null, transcript: null },
          decodability: { video: 'decodable', audio: 'decodable' },
          missing: false,
        },
      ],
    },
  };
}

function transcriptionStatus(installed: boolean): TranscriptionStatus {
  return {
    whisper: { path: '/bin/whisper-cli', problem: null },
    models: [
      {
        id: 'large-v3-turbo',
        fileName: 'ggml-large-v3-turbo.bin',
        size: 1_624_555_275,
        installed,
        custom: false,
        recommended: true,
        download: null,
      },
    ],
  } as unknown as TranscriptionStatus;
}

function answers(installed: boolean) {
  return (command: Command): unknown => {
    switch (command.kind) {
      case 'submitTask':
        return { kind: command.kind, task: taskOf({ id: 'task-2', projectId: IDS.project }) };
      case 'followUpTask':
        return { kind: command.kind, task: taskOf({ projectId: IDS.project }) };
      case 'getTranscriptionStatus':
        return { kind: command.kind, status: transcriptionStatus(installed) };
      default:
        return { kind: command.kind };
    }
  };
}

const clients: CoreClient[] = [];

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  setAiStatus(null);
  resetAiTasks();
  resetComposer();
  clearSelection();
});

function renderComposer(projectSession: ProjectSession, installed = true) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const answer = answers(installed);
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command) => Promise.resolve(answer(command) as never));
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(projectSession);
  act(() => {
    setAiStatus(READY);
  });
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <DesktopBridgeProvider bridge={idleBridge}>
          <ProjectStoreProvider store={store} sync={sync}>
            <ShortcutProvider registry={new ShortcutRegistry('other')}>
              <TooltipProvider>
                <TaskComposer />
              </TooltipProvider>
            </ShortcutProvider>
          </ProjectStoreProvider>
        </DesktopBridgeProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return { send };
}

/** task-1, finished a minute ago on the fixture sequence, with a session to resume. */
function recentTask(): Task {
  const finishedAt = Date.now() - 60_000;
  return taskOf({
    projectId: IDS.project,
    status: { kind: 'completed' },
    agentSessionId: 'session',
    scope: { sequenceId: IDS.sequence, range: null, tracks: [IDS.v1] },
    runs: [
      {
        runKind: 'initial',
        prompt: 'Coupe les blancs',
        status: { kind: 'completed' },
        queuedAt: finishedAt - 20_000,
        startedAt: finishedAt - 19_000,
        finishedAt,
      },
    ],
  });
}

async function send(text: string) {
  await userEvent.type(screen.getByRole('textbox', { name: 'Demande à l’IA' }), text);
  await userEvent.keyboard('{Control>}{Enter}{/Control}');
}

describe('ComposerNotices', () => {
  it('continues a task that just finished, with the composer’s scope', async () => {
    const { send: sent } = renderComposer(longSession());
    act(() => {
      setProjectTasks(IDS.project, [recentTask()]);
    });
    expect(screen.getByRole('checkbox', { name: /Continuer la tâche #1/ })).toBeChecked();
    await send('Et les doublons ?');
    expect(sent).toHaveBeenCalledWith({
      kind: 'followUpTask',
      projectId: IDS.project,
      taskId: 'task-1',
      prompt: 'Et les doublons ?',
      scope: { sequenceId: IDS.sequence, range: null, tracks: null },
    });
  });

  it('starts a new task once « continuer » is unticked', async () => {
    const { send: sent } = renderComposer(longSession());
    act(() => {
      setProjectTasks(IDS.project, [recentTask()]);
    });
    await userEvent.click(screen.getByRole('checkbox', { name: /Continuer la tâche #1/ }));
    await send('Autre chose');
    expect(sent).toHaveBeenCalledWith(expect.objectContaining({ kind: 'submitTask' }));
    expect(sent).not.toHaveBeenCalledWith(expect.objectContaining({ kind: 'followUpTask' }));
  });

  it('points out a small selection and widens the scope to the whole sequence', async () => {
    renderComposer(longSession());
    act(() => {
      selectItems([IDS.item]);
    });
    const notice = screen.getByText(/L’IA ne pourra modifier que la sélection/);
    expect(notice).toHaveTextContent('pistes track-v1');
    await userEvent.click(screen.getByRole('button', { name: 'Toute la séquence' }));
    expect(screen.queryByText(/L’IA ne pourra modifier que la sélection/)).toBeNull();
    expect(screen.getByText('Toute la séquence')).toBeInTheDocument();
  });

  it('warns when untranscribed media cannot be transcribed for lack of a model', async () => {
    renderComposer(speechSession(), false);
    expect(
      await screen.findByText(
        /« voix\.mp4 » n’est pas transcrit et aucun modèle de transcription n’est installé/,
      ),
    ).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Réglages' })).toBeInTheDocument();
  });

  it('says nothing about transcription when a model lets the agent transcribe', async () => {
    const { send: sent } = renderComposer(speechSession(), true);
    await waitFor(() => {
      expect(sent).toHaveBeenCalledWith({ kind: 'getTranscriptionStatus' });
    });
    expect(screen.queryByText(/n’est pas transcrit/)).toBeNull();
  });
});

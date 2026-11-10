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
  type TaskEventRecord,
} from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { ReactNode } from 'react';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { SimpleEffectBridge } from '../../../../electron/shared/bridge-api';
import { createI18n } from '../../../i18n';
import { openComposer, resetAiTasks, setProjectTasks } from '../../../shared/ai-tasks';
import { CoreClientProvider } from '../../../shared/core';
import { DesktopBridgeProvider } from '../../../shared/desktop';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { IDS, captionKind, session } from '../../../shared/project-store/testing/session-fixture';
import { ShortcutProvider, ShortcutRegistry } from '../../../shared/shortcuts';
import { useAiTaskSync } from '../hooks/use-ai-task-sync';
import { task as taskOf } from '../../../shared/ai-tasks/testing/task-fixture';
import { setAiStatus } from '../store/ai-status-store';
import { resetComposer } from '../store/composer-store';
import { AiStatusIndicator } from './AiStatusIndicator';
import { AiView } from './AiView';

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

/** A text « Abonne-toi » made by task-1 at 51–54 s on V2. */
const AI_TEXT: ItemEntity = {
  trackId: IDS.v2,
  id: 'item-ai',
  revision: 2,
  origin: { kind: 'aiTask', taskId: 'task-1' },
  timelineRange: { start: (51 * SECOND) as Flicks, duration: (3 * SECOND) as Flicks },
  linkGroup: null,
  kind: captionKind('Abonne-toi'),
};

function aiSession(): ProjectSession {
  const base = session();
  return {
    ...base,
    snapshot: {
      ...base.snapshot,
      tracks: base.snapshot.tracks.map((track) =>
        track.id === IDS.v2 ? { ...track, itemIds: [AI_TEXT.id] } : track,
      ),
      items: [...base.snapshot.items, AI_TEXT],
    },
  };
}

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
  getRuntimeInfo: () => new Promise(() => undefined),
  getThirdPartyNotices: () => Promise.resolve(null),
  showLicencesFolder: () => Promise.resolve(),
  onNotificationClick: () => () => undefined,
};

const clients: CoreClient[] = [];

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  setAiStatus(null);
  resetAiTasks();
  resetComposer();
});

function renderWithCore(ui: ReactNode, answer: (command: Command) => unknown = () => ({})) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command) => Promise.resolve(answer(command) as never));
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(aiSession());
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <DesktopBridgeProvider bridge={idleBridge}>
          <ProjectStoreProvider store={store} sync={sync}>
            <ShortcutProvider registry={new ShortcutRegistry('other')}>
              <TooltipProvider>{ui}</TooltipProvider>
            </ShortcutProvider>
          </ProjectStoreProvider>
        </DesktopBridgeProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return { send };
}

/** The AI tab with the editor's task sync mounted. */
function Panel() {
  useAiTaskSync();
  return <AiView />;
}

function seed(tasks: Task[]) {
  act(() => {
    setProjectTasks(IDS.project, tasks);
  });
}

const EVENTS: TaskEventRecord[] = [
  { sequence: 0, run: 0, at: 1, event: { kind: 'assistantText', text: 'J’ajoute le titre.' } },
  {
    sequence: 1,
    run: 0,
    at: 2,
    event: {
      kind: 'toolCallStarted',
      callId: 'call-1',
      tool: 'add_text',
      inputSummary: '{"text":"Abonne-toi","start":51,"end":54}',
    },
  },
  {
    sequence: 2,
    run: 0,
    at: 3,
    event: {
      kind: 'toolCallFinished',
      callId: 'call-1',
      tool: 'add_text',
      ok: true,
      outputSummary: '{"revision":4,"created":["item-ai"]}',
    },
  },
];

function answers(command: Command): unknown {
  switch (command.kind) {
    case 'getTaskEvents':
      return { kind: command.kind, events: EVENTS };
    case 'submitTask':
      return { kind: command.kind, task: taskOf({ id: 'task-2', projectId: IDS.project }) };
    case 'followUpTask':
      return { kind: command.kind, task: taskOf({ projectId: IDS.project }) };
    case 'previewTaskRevert':
      return {
        kind: command.kind,
        report: {
          taskId: command.taskId,
          reverted: true,
          revertedItems: ['item-1'],
          conflicts: [
            { kind: 'itemChanged', item: 'item-ai', taskRevision: 2, currentRevision: 3 },
          ],
        },
      };
    case 'revertTask':
      return {
        kind: command.kind,
        report: {
          taskId: command.taskId,
          reverted: true,
          revertedItems: ['item-1'],
          conflicts: [],
        },
      };
    default:
      return { kind: command.kind };
  }
}

describe('AiStatusIndicator', () => {
  it('shows the provider state and opens the AI settings', async () => {
    const onOpenSettings = vi.fn();
    renderWithCore(<AiStatusIndicator onOpenSettings={onOpenSettings} />);
    expect(screen.getByRole('button')).toHaveTextContent('Vérification de l’IA…');
    act(() => {
      setAiStatus({ ...READY, auth: { kind: 'loggedOut' } });
    });
    const button = screen.getByRole('button', { name: /Claude Code déconnecté/ });
    expect(button).toHaveAttribute('data-availability', 'loggedOut');
    await userEvent.hover(button);
    expect(
      (await screen.findAllByText('Ouvre un terminal, lance `claude`, puis tape `/login`.'))[0],
    ).toBeInTheDocument();
    await userEvent.click(button);
    expect(onOpenSettings).toHaveBeenCalledOnce();
    act(() => {
      setAiStatus(READY);
    });
    expect(screen.getByRole('button', { name: /Claude Code connecté/ })).toBeInTheDocument();
  });
});

describe('AiView', () => {
  it('explains a logged-out provider and refuses to send', async () => {
    act(() => {
      setAiStatus({ ...READY, auth: { kind: 'loggedOut' } });
    });
    renderWithCore(<Panel />, answers);
    expect(screen.getByText('Claude Code est déconnecté')).toBeInTheDocument();
    await userEvent.type(screen.getByRole('textbox', { name: 'Demande à l’IA' }), 'Coupe');
    expect(screen.getByRole('button', { name: 'Envoyer la demande' })).toBeDisabled();
    expect(screen.getByText('Aucune tâche IA')).toBeInTheDocument();
  });

  it('reads « de 51s à 54s » as the scope chip and submits it', async () => {
    act(() => {
      setAiStatus(READY);
    });
    const { send } = renderWithCore(<Panel />, answers);
    const field = screen.getByRole('textbox', { name: 'Demande à l’IA' });
    expect(screen.getByText('Toute la séquence')).toBeInTheDocument();
    await userEvent.type(field, 'de 51s à 54s mets un titre');
    expect(screen.getByText('00:51 → 00:54').closest('li')).toHaveAttribute('data-detected');
    await userEvent.keyboard('{Control>}{Enter}{/Control}');
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: 'submitTask',
        prompt: 'de 51s à 54s mets un titre',
        scope: {
          sequenceId: IDS.sequence,
          range: { start: 51 * SECOND, duration: 3 * SECOND },
          tracks: null,
        },
      }),
    );
    expect(field).toHaveValue('');
    await userEvent.type(field, '{ArrowUp}');
    expect(field).toHaveValue('de 51s à 54s mets un titre');
  });

  it('shows a running task live and cancels it', async () => {
    act(() => {
      setAiStatus(READY);
    });
    const { send } = renderWithCore(<Panel />, answers);
    seed([
      taskOf({
        projectId: IDS.project,
        prompt: 'De 51 s à 54 s mets « Abonne-toi »',
        status: { kind: 'running' },
        scope: {
          sequenceId: IDS.sequence,
          range: { start: (51 * SECOND) as Flicks, duration: (3 * SECOND) as Flicks },
          tracks: [IDS.v2],
        },
        usage: { inputTokens: 12_400, outputTokens: 300, costUsd: 0.02 },
      }),
    ]);
    const card = screen.getByRole('article');
    expect(within(card).getByText('En cours')).toBeInTheDocument();
    expect(within(card).getByText('00:51 → 00:54 · track-v2')).toBeInTheDocument();
    expect(
      await within(card).findByText('✓ add_text “Abonne-toi” 00:51→00:54'),
    ).toBeInTheDocument();
    expect(within(card).getByText(/12,7\s?k tokens/)).toBeInTheDocument();
    await userEvent.click(within(card).getByRole('button', { name: 'Annuler' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'cancelTask',
      projectId: IDS.project,
      taskId: 'task-1',
    });
  });

  it('explains a failure by its kind and shows the provider’s detail', () => {
    act(() => {
      setAiStatus(READY);
    });
    renderWithCore(<Panel />, answers);
    seed([
      taskOf({
        projectId: IDS.project,
        status: {
          kind: 'failed',
          failure: 'misconfigured',
          message: 'Claude Code refused the options SimpleEffect runs it with (…)',
        },
      }),
    ]);
    const alert = screen.getByRole('alert');
    expect(alert).toHaveTextContent(/choisissez l’exécutable claude lui-même dans Réglages → IA/);
    expect(alert).toHaveTextContent('Claude Code refused the options SimpleEffect runs it with');
  });

  it('lists the conflicts before reverting a task, then reverts the rest', async () => {
    act(() => {
      setAiStatus(READY);
    });
    const { send } = renderWithCore(<Panel />, answers);
    seed([taskOf({ projectId: IDS.project, status: { kind: 'completed' }, changesetId: 'cs-1' })]);
    await userEvent.click(screen.getByRole('button', { name: 'Annuler les modifications' }));
    const dialog = await screen.findByRole('alertdialog');
    expect(within(dialog).getByText('« Abonne-toi » a été modifié depuis')).toBeInTheDocument();
    expect(send).not.toHaveBeenCalledWith(expect.objectContaining({ kind: 'revertTask' }));
    await userEvent.click(within(dialog).getByRole('button', { name: 'Annuler le reste' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'revertTask',
      projectId: IDS.project,
      taskId: 'task-1',
    });
  });

  it('sends a follow-up from the inline field', async () => {
    act(() => {
      setAiStatus(READY);
    });
    const { send } = renderWithCore(<Panel />, answers);
    seed([
      taskOf({ projectId: IDS.project, status: { kind: 'completed' }, agentSessionId: 'session' }),
    ]);
    await userEvent.click(screen.getByRole('button', { name: 'Suite' }));
    await userEvent.type(
      screen.getByRole('textbox', { name: 'Suite de la tâche #1' }),
      'Plus gros',
    );
    await userEvent.click(screen.getByRole('button', { name: 'Envoyer la suite' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'followUpTask',
      projectId: IDS.project,
      taskId: 'task-1',
      prompt: 'Plus gros',
    });
  });

  it('« Refaire avec l’IA » resumes the session of the task that made the item', async () => {
    act(() => {
      setAiStatus(READY);
    });
    const { send } = renderWithCore(<Panel />, answers);
    seed([
      taskOf({
        projectId: IDS.project,
        status: { kind: 'completed' },
        agentSessionId: 'session',
        scope: {
          sequenceId: IDS.sequence,
          range: { start: (50 * SECOND) as Flicks, duration: (2 * SECOND) as Flicks },
          tracks: null,
        },
      }),
    ]);
    act(() => {
      openComposer(AI_TEXT.id);
    });
    expect(screen.getByText('Abonne-toi · 00:51 → 00:54')).toBeInTheDocument();
    expect(
      screen.getByRole('checkbox', { name: /Reprendre la session de la tâche #1/ }),
    ).toBeChecked();
    const field = screen.getByRole('textbox', { name: 'Demande à l’IA' });
    expect(field).toHaveFocus();
    await userEvent.type(field, 'En jaune');
    await userEvent.keyboard('{Control>}{Enter}{/Control}');
    expect(send).toHaveBeenCalledWith({
      kind: 'followUpTask',
      projectId: IDS.project,
      taskId: 'task-1',
      prompt: 'En jaune\n\nÉlément visé : « Abonne-toi » (item_id item-ai).',
      // The item (51–54 s) left the task's scope (50–52 s): the follow-up widens it.
      scope: {
        sequenceId: IDS.sequence,
        range: { start: 50 * SECOND, duration: 4 * SECOND },
        tracks: null,
      },
    });
  });
});

/**
 * @vitest-environment jsdom
 */
import { CoreClient, type AiStatus, type Command, type Task } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { CoreClientProvider } from '../../../shared/core';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { session } from '../../../shared/project-store/testing/session-fixture';
import { setAiStatus } from '../store/ai-status-store';
import { AiStatusIndicator } from './AiStatusIndicator';
import { AiView } from './AiView';

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

const clients: CoreClient[] = [];

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  setAiStatus(null);
});

function renderWithCore(ui: React.ReactNode, answer: (command: Command) => unknown = () => ({})) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command) => Promise.resolve(answer(command) as never));
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(session());
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <ProjectStoreProvider store={store} sync={sync}>
          <TooltipProvider>{ui}</TooltipProvider>
        </ProjectStoreProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return { send, client };
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
  const task: Task = {
    id: 'task-1',
    projectId: session().snapshot.project.id,
    prompt: 'Say hello',
    scope: { sequenceId: 'seq', range: null, tracks: null },
    provider: 'claude-code',
    model: null,
    effort: null,
    status: { kind: 'running' },
    agentSessionId: 'session-1',
    changesetId: null,
    createdAt: 1,
    updatedAt: 1,
    runs: [],
  };

  it('explains a logged-out provider without the development panel', () => {
    act(() => {
      setAiStatus({ ...READY, auth: { kind: 'loggedOut' } });
    });
    renderWithCore(<AiView devTools={false} />);
    expect(screen.getByText('Claude Code est déconnecté')).toBeInTheDocument();
    expect(screen.queryByRole('region', { name: /Tester une tâche/ })).not.toBeInTheDocument();
  });

  it('submits a test task, lists it and shows its events', async () => {
    act(() => {
      setAiStatus(READY);
    });
    const { send } = renderWithCore(<AiView devTools />, (command) => {
      switch (command.kind) {
        case 'submitTask':
          return { kind: command.kind, task };
        case 'listTasks':
          return { kind: command.kind, tasks: [] };
        case 'getTaskEvents':
          return {
            kind: command.kind,
            events: [
              {
                sequence: 0,
                run: 0,
                at: 1,
                event: { kind: 'assistantText', text: 'Bonjour' },
              },
            ],
          };
        default:
          return { kind: command.kind };
      }
    });
    const panel = screen.getByRole('region', { name: 'Tester une tâche (développement)' });
    await userEvent.type(within(panel).getByRole('textbox'), 'Say hello');
    await userEvent.click(within(panel).getByRole('button', { name: 'Envoyer' }));
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({ kind: 'submitTask', prompt: 'Say hello' }),
    );
    expect(await within(panel).findByText('Bonjour')).toBeInTheDocument();
  });
});

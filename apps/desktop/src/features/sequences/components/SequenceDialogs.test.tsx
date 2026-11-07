/**
 * @vitest-environment jsdom
 */
import { CoreClient, type Command } from '@se/protocol';
import { Toaster, TooltipProvider } from '@se/ui';
import { act, render, screen, waitFor, within } from '@testing-library/react';
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
import { reframeSession } from '../../../shared/project-store/testing/reframe-fixture';
import { IDS } from '../../../shared/project-store/testing/session-fixture';
import { estimateTextLayer } from '../services/text-extent';
import { useSequenceActions, type SequenceActionHandlers } from '../hooks/use-sequence-actions';
import { closeSequenceDialog, openSequenceDialog } from '../store/sequence-dialog-store';
import { SequenceDialogs } from './SequenceDialogs';

const FRAME_RATES = [
  { numerator: 25, denominator: 1 },
  { numerator: 30, denominator: 1 },
];

const clients: CoreClient[] = [];
afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  act(() => {
    closeSequenceDialog();
  });
});

function Harness(handlers: SequenceActionHandlers) {
  const actions = useSequenceActions(handlers);
  return <SequenceDialogs actions={actions} />;
}

/** The fixture session in 9:16, as the core's patch leaves it after a format change. */
function verticalSession() {
  const base = reframeSession();
  return {
    ...base,
    snapshot: {
      ...base.snapshot,
      revision: base.snapshot.revision + 1,
      sequences: base.snapshot.sequences.map((sequence) => ({
        ...sequence,
        format: { ...sequence.format, width: 1080, height: 1920 },
      })),
    },
  };
}

function renderDialogs() {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi.spyOn(client, 'send').mockImplementation((command: Command) => {
    switch (command.kind) {
      case 'setSequenceFormat':
        // The core publishes the patch before answering.
        sync.load(verticalSession());
        return Promise.resolve({ kind: command.kind } as never);
      case 'getNewProjectDefaults':
        return Promise.resolve({ kind: command.kind, frameRates: FRAME_RATES } as never);
      case 'duplicateSequence':
        return Promise.resolve({ kind: command.kind, sequenceId: 'sequence-2' } as never);
      default:
        return Promise.resolve({ kind: command.kind } as never);
    }
  });
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(reframeSession());
  const handlers = {
    onActivate: vi.fn(),
    onSelectItems: vi.fn(),
    measureText: estimateTextLayer,
  };
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <ProjectStoreProvider store={store} sync={sync}>
          <TooltipProvider>
            <Harness {...handlers} />
            <Toaster label="Notifications" />
          </TooltipProvider>
        </ProjectStoreProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return { send, handlers };
}

const sent = (send: ReturnType<typeof renderDialogs>['send'], kind: Command['kind']) =>
  send.mock.calls.map(([command]) => command).filter((command) => command.kind === kind);

describe('the sequence format dialog', () => {
  it('keeps the frame rate of a sequence holding items and fills a frame of another aspect', async () => {
    const user = userEvent.setup();
    const { send } = renderDialogs();
    act(() => {
      openSequenceDialog({ kind: 'format', sequenceId: IDS.sequence });
    });
    const dialog = await screen.findByRole('dialog', { name: 'Format de « Film »' });
    expect(within(dialog).getByRole('combobox', { name: 'Cadence' })).toBeDisabled();
    expect(within(dialog).getByText(/sa cadence ne peut plus changer/)).toBeInTheDocument();
    expect(within(dialog).getByRole('button', { name: 'Appliquer' })).toBeDisabled();

    await user.click(within(dialog).getByRole('radio', { name: '9:16' }));
    expect(within(dialog).getByRole('radio', { name: /Remplir le cadre/ })).toBeChecked();
    await user.click(within(dialog).getByRole('button', { name: 'Appliquer' }));

    await waitFor(() => {
      expect(sent(send, 'setSequenceFormat')).toHaveLength(1);
    });
    expect(sent(send, 'setSequenceFormat')[0]).toMatchObject({
      sequenceId: IDS.sequence,
      format: { width: 1080, height: 1920, frameRate: { numerator: 25, denominator: 1 } },
      adaptation: { clipReframe: 'fill', fitTextWidths: true },
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
  });

  it('sends the chosen clip policy and text option', async () => {
    const user = userEvent.setup();
    const { send } = renderDialogs();
    act(() => {
      openSequenceDialog({ kind: 'format', sequenceId: IDS.sequence });
    });
    const dialog = await screen.findByRole('dialog');
    await user.click(within(dialog).getByRole('radio', { name: '1:1' }));
    await user.click(within(dialog).getByRole('radio', { name: /Garder le cadrage/ }));
    await user.click(within(dialog).getByRole('checkbox', { name: /largeur des textes/ }));
    await user.click(within(dialog).getByRole('button', { name: 'Appliquer' }));
    await waitFor(() => {
      expect(sent(send, 'setSequenceFormat')[0]).toMatchObject({
        format: { width: 1080, height: 1080 },
        adaptation: { clipReframe: 'keep', fitTextWidths: false },
      });
    });
  });
});

describe('the warning about items leaving the frame', () => {
  it('names them and selects them on request', async () => {
    const user = userEvent.setup();
    const { handlers } = renderDialogs();
    act(() => {
      openSequenceDialog({ kind: 'format', sequenceId: IDS.sequence });
    });
    const dialog = await screen.findByRole('dialog');
    await user.click(within(dialog).getByRole('radio', { name: '9:16' }));
    await user.click(within(dialog).getByRole('checkbox', { name: /largeur des textes/ }));
    await user.click(within(dialog).getByRole('button', { name: 'Appliquer' }));
    // Sonner renders the toast in its list and in a live region: take the first.
    expect((await screen.findAllByText('1 élément sort du cadre'))[0]).toBeInTheDocument();
    expect(screen.getAllByText('« Un titre très large »')[0]).toBeInTheDocument();
    const select = screen.getAllByRole('button', { name: 'Sélectionner' })[0];
    if (select === undefined) throw new Error('no « Sélectionner » button');
    await user.click(select);
    expect(handlers.onSelectItems).toHaveBeenCalledWith(IDS.sequence, ['title']);
  });
});

describe('creating a version', () => {
  it('copies the sequence into 9:16 with clips filling the frame and says it is independent', async () => {
    const user = userEvent.setup();
    const { send, handlers } = renderDialogs();
    act(() => {
      openSequenceDialog({ kind: 'version', sequenceId: IDS.sequence });
    });
    const dialog = await screen.findByRole('dialog', { name: 'Créer une version de « Film »' });
    expect(within(dialog).getByText(/indépendante/)).toBeInTheDocument();
    expect(within(dialog).getByRole('textbox', { name: 'Nom de la version' })).toHaveValue(
      'Film 9:16',
    );
    expect(within(dialog).getByRole('radio', { name: /Remplir le cadre/ })).toBeChecked();
    await user.click(within(dialog).getByRole('button', { name: 'Créer la version' }));

    await waitFor(() => {
      expect(sent(send, 'duplicateSequence')).toHaveLength(1);
    });
    expect(sent(send, 'duplicateSequence')[0]).toMatchObject({
      sequenceId: IDS.sequence,
      name: 'Film 9:16',
      format: { width: 1080, height: 1920 },
      adaptation: { clipReframe: 'fill', fitTextWidths: true },
    });
    expect(handlers.onActivate).toHaveBeenCalledWith('sequence-2');
    expect((await screen.findAllByText('Version « Film 9:16 » créée'))[0]).toBeInTheDocument();
  });
});

describe('removing a sequence', () => {
  it('asks for confirmation first', async () => {
    const user = userEvent.setup();
    const { send } = renderDialogs();
    act(() => {
      openSequenceDialog({ kind: 'remove', sequenceId: IDS.sequence });
    });
    const dialog = await screen.findByRole('alertdialog', { name: 'Supprimer « Film » ?' });
    expect(sent(send, 'removeSequence')).toHaveLength(0);
    await user.click(within(dialog).getByRole('button', { name: 'Supprimer' }));
    await waitFor(() => {
      expect(sent(send, 'removeSequence')).toEqual([
        { kind: 'removeSequence', projectId: IDS.project, sequenceId: IDS.sequence },
      ]);
    });
  });
});

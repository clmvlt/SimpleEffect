/**
 * @vitest-environment jsdom
 */
import { CoreClient, type Command, type Flicks, type ProjectSession } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, fireEvent, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { CoreClientProvider } from '../../../shared/core';
import { DesktopBridgeProvider } from '../../../shared/desktop';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { SECOND } from '../../../shared/project-store/testing/graphics-fixture';
import {
  badgeTemplate,
  templateSession,
} from '../../../shared/project-store/testing/template-fixture';
import { clearSelection, selectionStore, setPlayheadTime } from '../../timeline';
import { closeTemplateCode, codeDialogStore } from '../store/code-dialog-store';
import { closeLibraryDialog } from '../store/library-dialog-store';
import { resetLibrary } from '../store/library-store';
import { libraryEntry, libraryStyle } from '../testing/library-fixture';
import { LibraryInfoDialog } from './LibraryInfoDialog';
import { TemplateBrowser } from './TemplateBrowser';

const clients: CoreClient[] = [];
afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  clearSelection();
  closeTemplateCode();
  closeLibraryDialog();
  resetLibrary();
  setPlayheadTime('sequence-1', 0 as Flicks);
});

/** A session whose template `badge` is a copy of version 1 of the library entry `entry-1`. */
function linkedSession(): ProjectSession {
  const base = templateSession();
  const badge = { ...badgeTemplate(), library: { templateId: 'entry-1', version: 1 } };
  return {
    ...base,
    snapshot: { ...base.snapshot, project: { ...base.snapshot.project, templates: [badge] } },
  };
}

function answer(command: Command) {
  switch (command.kind) {
    case 'listLibraryTemplates':
      return { kind: command.kind, templates: [libraryEntry({ version: 2 }), libraryStyle()] };
    case 'addGraphicItem':
      return { kind: command.kind, itemId: 'badge-2', trackId: 'track-v2' };
    case 'insertLibraryTemplate':
      return { kind: command.kind, templateId: 'copy', itemId: 'copy-1', trackId: 'track-v2' };
    case 'updateTemplateFromLibrary':
      return { kind: command.kind, version: 3, libraryVersion: 2, reset: [] };
    case 'saveTemplateToLibrary':
      return { kind: command.kind, template: libraryEntry({ version: 3 }) };
    default:
      return { kind: command.kind };
  }
}

function renderBrowser(session = linkedSession()) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command: Command) => Promise.resolve(answer(command) as never));
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(session);
  const bridge = { chooseSaveFile: vi.fn(), chooseFiles: vi.fn() };
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <DesktopBridgeProvider bridge={bridge as never}>
          <ProjectStoreProvider store={store} sync={sync}>
            <TooltipProvider>
              <TemplateBrowser />
              <LibraryInfoDialog />
            </TooltipProvider>
          </ProjectStoreProvider>
        </DesktopBridgeProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return { send, bridge };
}

function card(key: string): HTMLElement {
  const element = document.querySelector<HTMLElement>(`[data-browser-entry="${key}"]`);
  if (element === null) throw new Error(`card ${key} is shown`);
  return element;
}

describe('the template browser', () => {
  it('shows the project’s templates and the library, with the update the library offers', async () => {
    renderBrowser();
    await screen.findByText('Sobre');
    expect(within(card('project:badge')).getByText('Badge')).toBeInTheDocument();
    expect(within(card('project:badge')).getByText('v2 disponible')).toBeInTheDocument();
    expect(within(card('library:entry-1')).getByText('Bibliothèque')).toBeInTheDocument();
    fireEvent.click(screen.getByRole('radio', { name: 'Projet' }));
    expect(screen.queryByText('Sobre')).not.toBeInTheDocument();
    fireEvent.click(screen.getByRole('radio', { name: 'Bibliothèque' }));
    fireEvent.click(screen.getByRole('radio', { name: 'Sous-titres' }));
    expect(screen.queryByText('Bandeau')).not.toBeInTheDocument();
    expect(screen.getByText('Sobre')).toBeInTheDocument();
  });

  it('inserts project and library templates at the playhead, then selects them', async () => {
    const { send } = renderBrowser();
    await screen.findByText('Bandeau');
    act(() => {
      setPlayheadTime('sequence-1', (2 * SECOND) as Flicks);
    });
    fireEvent.click(
      within(card('project:badge')).getByRole('button', { name: 'Insérer à la tête de lecture' }),
    );
    expect(send).toHaveBeenCalledWith({
      kind: 'addGraphicItem',
      projectId: 'project-1',
      sequenceId: 'sequence-1',
      at: 2 * SECOND,
      duration: 3 * SECOND,
      graphic: { kind: 'template', templateId: 'badge' },
    });
    await vi.waitFor(() => {
      expect(selectionStore.getState().itemIds).toEqual(['badge-2']);
    });
    fireEvent.click(screen.getByRole('radio', { name: 'Bibliothèque' }));
    fireEvent.click(
      within(card('library:entry-1')).getByRole('button', { name: 'Insérer à la tête de lecture' }),
    );
    expect(send).toHaveBeenCalledWith({
      kind: 'insertLibraryTemplate',
      projectId: 'project-1',
      templateId: 'entry-1',
      sequenceId: 'sequence-1',
      trackId: null,
      start: 2 * SECOND,
    });
  });

  it('updates a copy on request and saves a template to the library', async () => {
    const user = userEvent.setup();
    const { send } = renderBrowser();
    await screen.findByText('Bandeau');
    await user.click(
      within(card('project:badge')).getByRole('button', { name: 'Actions de « Badge »' }),
    );
    await user.click(
      await screen.findByRole('menuitem', { name: 'Mettre à jour vers la version 2' }),
    );
    expect(send).toHaveBeenCalledWith({
      kind: 'updateTemplateFromLibrary',
      projectId: 'project-1',
      templateId: 'badge',
    });
    await user.click(
      within(card('project:badge')).getByRole('button', { name: 'Actions de « Badge »' }),
    );
    await user.click(
      await screen.findByRole('menuitem', {
        name: 'Enregistrer une nouvelle version dans la bibliothèque…',
      }),
    );
    // A copy of « Bandeau » v2: the new version starts from the library entry's information.
    const dialog = await screen.findByRole('dialog', {
      name: 'Enregistrer la version 3 dans la bibliothèque',
    });
    expect(within(dialog).getByLabelText('Nom')).toHaveValue('Bandeau');
    await user.type(within(dialog).getByLabelText('Mots-clés'), ', Shorts');
    await user.click(within(dialog).getByRole('button', { name: 'Enregistrer' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'saveTemplateToLibrary',
      projectId: 'project-1',
      templateId: 'badge',
      info: {
        name: 'Bandeau',
        description: 'Nom et fonction',
        tags: ['interview', 'Shorts'],
        category: 'other',
      },
    });
  });

  it('opens the code and explains how to get templates when there is none', async () => {
    const user = userEvent.setup();
    renderBrowser();
    await screen.findByText('Bandeau');
    await user.click(
      within(card('project:badge')).getByRole('button', { name: 'Actions de « Badge »' }),
    );
    await user.click(await screen.findByRole('menuitem', { name: 'Voir le code' }));
    expect(codeDialogStore.getState().templateId).toBe('badge');
    fireEvent.change(screen.getByRole('searchbox', { name: 'Rechercher un modèle' }), {
      target: { value: 'introuvable' },
    });
    expect(screen.getByText('Aucun modèle ne correspond')).toBeInTheDocument();
  });
});

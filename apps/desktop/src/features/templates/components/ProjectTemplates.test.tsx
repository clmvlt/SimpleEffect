/**
 * @vitest-environment jsdom
 */
import { CoreClient, type Command, type Flicks } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, fireEvent, render, screen } from '@testing-library/react';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { CoreClientProvider } from '../../../shared/core';
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
import { ProjectTemplates } from './ProjectTemplates';

const clients: CoreClient[] = [];
afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  clearSelection();
  closeTemplateCode();
  setPlayheadTime('sequence-1', 0 as Flicks);
});

function renderList(session = templateSession()) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command: Command) =>
      Promise.resolve(
        (command.kind === 'addGraphicItem'
          ? { kind: 'addGraphicItem', itemId: 'badge-2', trackId: 'track-v2' }
          : { kind: command.kind }) as never,
      ),
    );
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(session);
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <ProjectStoreProvider store={store} sync={sync}>
          <TooltipProvider>
            <ProjectTemplates />
          </TooltipProvider>
        </ProjectStoreProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return send;
}

describe('the project templates list', () => {
  it('lists the templates; a used one cannot be removed', () => {
    renderList();
    expect(screen.getByText('Badge')).toBeInTheDocument();
    expect(screen.getByText('A corner badge')).toBeInTheDocument();
    expect(screen.getByText('v2')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Supprimer le modèle' })).toBeDisabled();
  });

  it('inserts an instance at the playhead for the template’s length, then selects it', async () => {
    const send = renderList();
    act(() => {
      setPlayheadTime('sequence-1', (2 * SECOND) as Flicks);
    });
    fireEvent.click(screen.getByRole('button', { name: 'Insérer à la tête de lecture' }));
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
  });

  it('removes an unused template and opens the code', () => {
    const base = templateSession();
    const unused = {
      ...base,
      snapshot: {
        ...base.snapshot,
        tracks: base.snapshot.tracks.map((track) => ({ ...track, itemIds: [] })),
        items: [],
      },
    };
    const send = renderList(unused);
    fireEvent.click(screen.getByRole('button', { name: 'Supprimer le modèle' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'removeTemplate',
      projectId: 'project-1',
      templateId: badgeTemplate().id,
    });
    fireEvent.click(screen.getByRole('button', { name: 'Voir le code' }));
    expect(codeDialogStore.getState().templateId).toBe('badge');
  });

  it('explains how to get templates when the project has none', () => {
    const base = templateSession();
    renderList({
      ...base,
      snapshot: { ...base.snapshot, project: { ...base.snapshot.project, templates: [] } },
    });
    expect(screen.getByText('Aucun modèle d’animation')).toBeInTheDocument();
  });
});

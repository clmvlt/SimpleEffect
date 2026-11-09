/**
 * @vitest-environment jsdom
 */
import { CoreClient, type Command, type Flicks } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { clearPropertyPreview, propertyPreviewStore } from '../../../shared/animation';
import { composerRequestStore } from '../../../shared/ai-tasks';
import { CoreClientProvider } from '../../../shared/core';
import { resetFontCatalog } from '../../../shared/fonts';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { templateSession } from '../../../shared/project-store/testing/template-fixture';
import { publishTemplateIssues } from '../../../shared/templates';
import { TemplateCodeDialog, closeTemplateCode } from '../../templates';
import { clearSelection, selectItems, setPlayheadTime } from '../../timeline';
import { InspectorView } from './InspectorView';

const clients: CoreClient[] = [];
afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  clearSelection();
  clearPropertyPreview();
  resetFontCatalog();
  closeTemplateCode();
  publishTemplateIssues([]);
  setPlayheadTime('sequence-1', 0 as Flicks);
});

function renderInspector() {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command: Command) =>
      Promise.resolve(
        (command.kind === 'listFonts'
          ? { kind: 'listFonts', families: [] }
          : { kind: command.kind }) as never,
      ),
    );
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(templateSession());
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <ProjectStoreProvider store={store} sync={sync}>
          <TooltipProvider>
            <InspectorView />
            <TemplateCodeDialog />
          </TooltipProvider>
        </ProjectStoreProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  act(() => {
    selectItems(['badge-1']);
  });
  return send;
}

const updates = (send: ReturnType<typeof renderInspector>) =>
  send.mock.calls
    .map(([command]) => command)
    .flatMap((command) => (command.kind === 'updateItems' ? command.updates : []));

describe('the generated template inspector', () => {
  it('shows one control per parameter with the instance’s values and the defaults', () => {
    renderInspector();
    expect(screen.getByRole('button', { name: 'Modèle : Badge' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Transformation' })).toBeInTheDocument();
    expect(screen.getByLabelText('Texte')).toHaveValue('PROMO');
    expect(screen.getByRole('spinbutton', { name: 'Taille' })).toHaveAttribute(
      'aria-valuenow',
      '60',
    );
    expect(screen.getByRole('switch', { name: 'Visible' })).toBeChecked();
    expect(screen.getByRole('combobox', { name: 'Coin' })).toHaveTextContent('En haut à droite');
    expect(screen.getByRole('combobox', { name: 'Logo' })).toHaveTextContent('Aucune image');
  });

  it('previews a typed text and commits it as one template patch', async () => {
    const send = renderInspector();
    const field = screen.getByLabelText('Texte');
    await userEvent.clear(field);
    await userEvent.type(field, 'SOLDES');
    expect(propertyPreviewStore.getState().templateParams['badge-1']).toEqual({
      label: { kind: 'text', value: 'SOLDES' },
    });
    fireEvent.blur(field);
    expect(updates(send)).toEqual([
      {
        itemId: 'badge-1',
        patch: {
          kind: 'motionTemplate',
          templateId: null,
          params: { label: { kind: 'text', value: 'SOLDES' } },
        },
      },
    ]);
  });

  it('sets numbers, flags and options, and keyframes numbers at the playhead', async () => {
    const send = renderInspector();
    const size = screen.getByRole('spinbutton', { name: 'Taille' });
    size.focus();
    await userEvent.keyboard('{ArrowUp}');
    fireEvent.click(screen.getByRole('switch', { name: 'Visible' }));
    fireEvent.click(screen.getByRole('button', { name: 'Image clé : Taille' }));
    const params = updates(send).map((update) =>
      update.patch.kind === 'motionTemplate' ? update.patch.params : null,
    );
    expect(params).toEqual([
      { size: { kind: 'number', value: { kind: 'static', value: 61 } } },
      { shown: { kind: 'boolean', value: false } },
      {
        size: {
          kind: 'number',
          value: {
            kind: 'keyframed',
            keyframes: [{ time: 0, value: 60, easing: { kind: 'linear' } }],
          },
        },
      },
    ]);
  });

  it('shows the template’s failure in the frame on screen', () => {
    renderInspector();
    act(() => {
      publishTemplateIssues([
        { itemId: 'badge-1', templateId: 'badge', message: 'TypeError: boom', line: 4 },
      ]);
    });
    expect(screen.getByRole('alert')).toHaveTextContent(
      'Ce modèle a échoué (ligne 4) : TypeError: boom',
    );
  });

  it('opens the coloured code and asks the AI for a change', () => {
    renderInspector();
    fireEvent.click(screen.getByRole('button', { name: 'Voir le code' }));
    const dialog = screen.getByRole('dialog', { name: 'Badge — version 2' });
    expect(dialog).toHaveTextContent('export function render(ctx, time, p) {');
    expect(dialog.querySelector('.text-ai')).toHaveTextContent('export');
    fireEvent.keyDown(dialog, { key: 'Escape' });
    fireEvent.click(screen.getByRole('button', { name: 'Demander une modification à l’IA' }));
    expect(composerRequestStore.getState().request).toMatchObject({
      itemId: 'badge-1',
      draft: 'Modifie le modèle « Badge » : ',
    });
  });
});

/**
 * @vitest-environment jsdom
 */
import { CoreClient, type BrandKit, type Command, type LibraryBrandKit } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { fireEvent, render, screen, within } from '@testing-library/react';
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
import { resetBrandKits } from '../store/brand-kits-store';
import { kitSession, libraryKit, projectCopy } from '../testing/kit-fixture';
import { BrandKitButton } from './BrandKitButton';
import { BrandKitsSection } from './BrandKitsSection';

const clients: CoreClient[] = [];
afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  resetBrandKits();
});

function answer(command: Command, kits: readonly LibraryBrandKit[]) {
  switch (command.kind) {
    case 'listBrandKits':
      return { kind: command.kind, kits };
    case 'listLibraryTemplates':
      return { kind: command.kind, templates: [] };
    case 'saveBrandKit':
      return { kind: command.kind, kit: { ...command.kit, version: command.kit.version + 1 } };
    case 'createBrandKit':
      return { kind: command.kind, kit: libraryKit(1, { id: 'kit-2', name: command.name }) };
    case 'addProjectBrandKitToLibrary':
      return { kind: command.kind, kit: libraryKit(1, { id: 'kit-3' }) };
    default:
      return { kind: command.kind };
  }
}

function renderWith(
  ui: React.ReactNode,
  copy: BrandKit | null,
  kits: readonly LibraryBrandKit[] = [libraryKit(3), libraryKit(1, { id: 'kit-9', name: 'Perso' })],
) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command: Command) => Promise.resolve(answer(command, kits) as never));
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(kitSession(copy));
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <DesktopBridgeProvider bridge={{ chooseFiles: vi.fn() } as never}>
          <ProjectStoreProvider store={store} sync={sync}>
            <TooltipProvider>{ui}</TooltipProvider>
          </ProjectStoreProvider>
        </DesktopBridgeProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return send;
}

describe('the brand kit menu', () => {
  it('shows the project’s kit, offers the newer version and every kit of the library', async () => {
    const user = userEvent.setup();
    const onManage = vi.fn();
    const send = renderWith(
      <BrandKitButton projectId="project-1" onManage={onManage} />,
      projectCopy(libraryKit(3), 2),
    );
    await user.click(screen.getByRole('button', { name: 'Brand kit du projet' }));
    expect(await screen.findByRole('menuitemradio', { name: 'Acme' })).toBeChecked();
    await user.click(screen.getByRole('menuitem', { name: 'Mettre à jour vers la version 3' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'setProjectBrandKit',
      projectId: 'project-1',
      kitId: 'kit-1',
    });
    await user.click(screen.getByRole('button', { name: 'Brand kit du projet' }));
    await user.click(await screen.findByRole('menuitemradio', { name: 'Aucun' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'setProjectBrandKit',
      projectId: 'project-1',
      kitId: null,
    });
    await user.click(screen.getByRole('button', { name: 'Brand kit du projet' }));
    await user.click(await screen.findByRole('menuitem', { name: 'Gérer les brand kits…' }));
    expect(onManage).toHaveBeenCalledOnce();
  });

  it('offers to add a kit made elsewhere to the library', async () => {
    const user = userEvent.setup();
    const send = renderWith(
      <BrandKitButton projectId="project-1" onManage={vi.fn()} />,
      projectCopy(libraryKit(1, { id: 'kit-elsewhere' }), 1),
    );
    await user.click(screen.getByRole('button', { name: 'Brand kit du projet' }));
    await user.click(await screen.findByRole('menuitem', { name: 'Ajouter à mes brand kits' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'addProjectBrandKitToLibrary',
      projectId: 'project-1',
    });
  });
});

describe('the brand kits settings', () => {
  it('edits a kit as a draft and saves it as a new version', async () => {
    const user = userEvent.setup();
    const send = renderWith(<BrandKitsSection projectId="project-1" />, null);
    const name = await screen.findByRole('textbox', { name: 'Nom' });
    const save = screen.getByRole('button', { name: 'Enregistrer le kit' });
    expect(save).toBeDisabled();
    await user.clear(name);
    await user.type(name, 'Acme Studio');
    expect(screen.getByText('Modifications non enregistrées')).toBeInTheDocument();
    await user.click(save);
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({
        kind: 'saveBrandKit',
        kit: expect.objectContaining({ id: 'kit-1', name: 'Acme Studio' }) as unknown,
      }),
    );
    await vi.waitFor(() => {
      expect(screen.getByText('Version 4')).toBeInTheDocument();
    });
  });

  it('lists the kits, creates one and chooses one for the project', async () => {
    const user = userEvent.setup();
    const send = renderWith(<BrandKitsSection projectId="project-1" />, null);
    const list = await screen.findByRole('navigation', { name: 'Brand kits' });
    fireEvent.click(within(list).getByRole('button', { name: 'Perso' }));
    expect(screen.getByRole('textbox', { name: 'Nom' })).toHaveValue('Perso');
    await user.click(screen.getByRole('button', { name: 'Utiliser dans ce projet' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'setProjectBrandKit',
      projectId: 'project-1',
      kitId: 'kit-9',
    });
    await user.click(within(list).getByRole('button', { name: 'Nouveau brand kit' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'createBrandKit',
      name: 'Mon brand kit',
      copyOf: null,
    });
  });
});

/**
 * @vitest-environment jsdom
 */
import { CoreClient } from '@se/protocol';
import type { DockLayoutStorage } from '@se/ui';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { createMemoryRouter, RouterProvider } from 'react-router';
import { afterEach, describe, expect, it } from 'vitest';

import { createI18n } from '../i18n';
import { AppProviders } from './AppProviders';
import { createAppRoutes } from './routes';

const clients: CoreClient[] = [];

/** A real client whose endpoint never resolves: it stays « connecting » and opens no socket. */
function idleCoreClient(): CoreClient {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  return client;
}

function memoryStorage(): DockLayoutStorage & { entries: Map<string, string> } {
  const entries = new Map<string, string>();
  return {
    entries,
    getItem: (key) => entries.get(key) ?? null,
    setItem: (key, value) => {
      entries.set(key, value);
    },
  };
}

function renderApp(path: string, layoutStorage: DockLayoutStorage = memoryStorage()) {
  const router = createMemoryRouter(createAppRoutes({ layoutStorage }), { initialEntries: [path] });
  return render(
    <AppProviders coreClient={idleCoreClient()} i18n={createI18n()}>
      <RouterProvider router={router} />
    </AppProviders>,
  );
}

afterEach(() => {
  for (const client of clients.splice(0)) {
    client.close();
  }
});

describe('app shell', () => {
  it('shows the empty home screen with a disabled « Nouveau projet »', () => {
    renderApp('/');
    expect(screen.getByRole('heading', { name: 'Accueil' })).toBeInTheDocument();
    expect(screen.getByText('Aucun projet')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Nouveau projet' })).toBeDisabled();
  });

  it('opens the editor from the development preview link', async () => {
    const user = userEvent.setup();
    renderApp('/');
    await user.click(screen.getByRole('link', { name: /aperçu de développement/ }));
    expect(screen.getByRole('region', { name: 'Médias' })).toBeInTheDocument();
  });

  it('lays out the editor panels with explicit empty states', async () => {
    const user = userEvent.setup();
    renderApp('/project/demo');

    expect(
      within(screen.getByRole('region', { name: 'Médias' })).getByText('Aucun média'),
    ).toBeInTheDocument();
    expect(
      within(screen.getByRole('region', { name: 'Viewer' })).getByText('Aucune séquence ouverte'),
    ).toBeInTheDocument();
    expect(
      within(screen.getByRole('region', { name: 'Timeline' })).getByText('Timeline vide'),
    ).toBeInTheDocument();

    const side = screen.getByRole('region', { name: 'Inspecteur et IA' });
    expect(within(side).getByText('Aucune sélection')).toBeInTheDocument();
    // Keyboard, not a click: in jsdom every element measures 0 px, so the panel library takes any
    // press for a resize grab on a panel edge and swallows it.
    within(side).getByRole('tab', { name: 'Inspecteur' }).focus();
    await user.keyboard('{ArrowRight}');
    expect(within(side).getByText('Assistant IA non configuré')).toBeInTheDocument();

    expect(screen.getAllByRole('separator')).toHaveLength(3);
  });

  it('restores saved panel sizes', () => {
    const storage = memoryStorage();
    storage.setItem(
      'dock-layout.v1.editor.columns',
      JSON.stringify({ media: 35, viewer: 40, side: 25 }),
    );
    renderApp('/project/demo', storage);
    const mediaHandle = screen.getByRole('separator', {
      name: 'Redimensionner les panneaux Médias et Viewer',
    });
    expect(Number(mediaHandle.getAttribute('aria-valuenow'))).toBeCloseTo(35);
  });

  it('shows the core connection, AI availability and version in the status bar', () => {
    renderApp('/');
    const statusBar = screen.getByRole('contentinfo', { name: 'Barre d’état' });
    expect(within(statusBar).getByRole('status')).toHaveTextContent('Connexion au cœur…');
    expect(statusBar).toHaveTextContent('IA non configurée');
    expect(statusBar).toHaveTextContent('Version 0.0.0-test');
  });
});

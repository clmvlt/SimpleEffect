/**
 * @vitest-environment jsdom
 */
import { CoreClient } from '@se/protocol';
import type { DockLayoutStorage } from '@se/ui';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { createMemoryRouter, RouterProvider } from 'react-router';
import { afterEach, describe, expect, it } from 'vitest';

import type { SimpleEffectBridge } from '../shared/desktop';
import { createI18n } from '../i18n';
import { connectProjectSync, createProjectStore } from '../shared/project-store';
import { IDS, session } from '../shared/project-store/testing/session-fixture';
import { ShortcutRegistry } from '../shared/shortcuts';
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

/** A bridge whose native dialogs are never answered. */
const idleBridge: SimpleEffectBridge = {
  getCoreConnection: () => new Promise(() => undefined),
  chooseFolder: () => Promise.resolve(null),
  chooseFiles: () => Promise.resolve([]),
  showInFolder: () => Promise.resolve(),
  getPathForFile: () => '',
};

/** Renders the app at `path`; `loaded` puts the fixture project in the store first. */
function renderApp(path: string, { layoutStorage = memoryStorage(), loaded = false } = {}) {
  const coreClient = idleCoreClient();
  const projectStore = createProjectStore();
  const projectSync = connectProjectSync(coreClient, projectStore);
  if (loaded) projectSync.load(session());
  const router = createMemoryRouter(createAppRoutes({ layoutStorage }), { initialEntries: [path] });
  return render(
    <AppProviders
      coreClient={coreClient}
      i18n={createI18n()}
      bridge={idleBridge}
      projectStore={projectStore}
      projectSync={projectSync}
      shortcuts={new ShortcutRegistry('other')}
    >
      <RouterProvider router={router} />
    </AppProviders>,
  );
}

const EDITOR_PATH = `/project/${IDS.project}`;

afterEach(() => {
  for (const client of clients.splice(0)) {
    client.close();
  }
});

describe('app shell', () => {
  it('shows the home screen loading the recent projects while the core connects', () => {
    renderApp('/');
    expect(screen.getByRole('heading', { name: 'Accueil' })).toBeInTheDocument();
    expect(screen.getByRole('status', { name: 'Chargement des projets récents…' })).toBeVisible();
    expect(screen.getByRole('button', { name: 'Nouveau projet' })).toBeEnabled();
    expect(screen.getByRole('button', { name: 'Ouvrir un projet' })).toBeEnabled();
  });

  it('waits for the project of the URL before showing the editor', () => {
    renderApp('/project/unknown');
    expect(screen.getByRole('status', { name: 'Ouverture du projet…' })).toBeVisible();
    expect(screen.queryByRole('region', { name: 'Médias' })).not.toBeInTheDocument();
  });

  it('shows the project name, its sequence and disabled undo in the editor header', () => {
    renderApp(EDITOR_PATH, { loaded: true });
    const header = screen.getByRole('banner', { name: 'Projet' });
    expect(within(header).getByRole('heading', { name: 'Film' })).toBeInTheDocument();
    expect(header).toHaveTextContent('Séquence Film');
    expect(within(header).getByRole('button', { name: 'Annuler' })).toBeDisabled();
    expect(within(header).getByRole('button', { name: 'Accueil' })).toBeEnabled();
  });

  it('lays out the editor panels, the timeline showing the active sequence', async () => {
    const user = userEvent.setup();
    renderApp(EDITOR_PATH, { loaded: true });

    expect(
      within(screen.getByRole('region', { name: 'Médias' })).getByText('Aucun média'),
    ).toBeInTheDocument();
    const viewer = screen.getByRole('region', { name: 'Viewer' });
    expect(within(viewer).getByText('1920 × 1080 · 25 i/s')).toBeInTheDocument();
    // jsdom has no WebCodecs: the viewer explains it instead of loading the engine.
    expect(await within(viewer).findByText('Aperçu indisponible')).toBeInTheDocument();
    const timeline = screen.getByRole('region', { name: 'Timeline' });
    expect(within(timeline).getByRole('group', { name: 'Piste track-v1' })).toBeInTheDocument();
    expect(within(timeline).getByRole('option', { name: 'item-1' })).toBeInTheDocument();

    const side = screen.getByRole('region', { name: 'Inspecteur et IA' });
    expect(within(side).getByText('Aucune sélection')).toBeInTheDocument();
    // Keyboard, not a click: in jsdom every element measures 0 px, so the panel library takes any
    // press for a resize grab on a panel edge and swallows it.
    within(side).getByRole('tab', { name: 'Inspecteur' }).focus();
    await user.keyboard('{ArrowRight}');
    expect(within(side).getByText('Assistant IA non configuré')).toBeInTheDocument();

    expect(screen.getAllByRole('separator', { name: /^Redimensionner/ })).toHaveLength(3);
  });

  it('restores saved panel sizes', () => {
    const storage = memoryStorage();
    storage.setItem(
      'dock-layout.v1.editor.columns',
      JSON.stringify({ media: 35, viewer: 40, side: 25 }),
    );
    renderApp(EDITOR_PATH, { layoutStorage: storage, loaded: true });
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

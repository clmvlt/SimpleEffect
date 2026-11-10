/**
 * @vitest-environment jsdom
 */
import { CoreClient } from '@se/protocol';
import type { DockLayoutStorage } from '@se/ui';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { createMemoryRouter, RouterProvider } from 'react-router';
import { afterEach, describe, expect, it } from 'vitest';

import { aboutDialogStore } from '../features/about';
import { ONBOARDING_DONE_KEY, onboardingStore } from '../features/onboarding';
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
  showNotification: () => Promise.resolve(),
  resolveFilePath: ({ folder, fileName }) =>
    Promise.resolve({ path: `${folder}/${fileName}`, exists: false }),
  setExportActivity: () => Promise.resolve(),
  getRuntimeInfo: () => new Promise(() => undefined),
  getThirdPartyNotices: () => Promise.resolve(null),
  showLicencesFolder: () => Promise.resolve(),
  onNotificationClick: () => () => undefined,
  chooseFile: () => Promise.resolve(null),
  chooseSaveFile: () => Promise.resolve(null),
  openExternal: () => Promise.resolve(),
};

/** Renders the app at `path`; `loaded` puts the fixture project in the store first. */
/** Preferences of a machine that already went through the first-run wizard. */
function onboardedPreferences() {
  const storage = memoryStorage();
  storage.setItem(ONBOARDING_DONE_KEY, '2026-11-01T00:00:00.000Z');
  return storage;
}

function renderApp(
  path: string,
  { layoutStorage = memoryStorage(), loaded = false, preferences = onboardedPreferences() } = {},
) {
  const coreClient = idleCoreClient();
  const projectStore = createProjectStore();
  const projectSync = connectProjectSync(coreClient, projectStore);
  if (loaded) projectSync.load(session());
  const router = createMemoryRouter(createAppRoutes({ layoutStorage, preferences }), {
    initialEntries: [path],
  });
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
  aboutDialogStore.setState({ open: false, tab: 'application' });
  onboardingStore.setState({ open: false, step: 'welcome' });
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
    expect(within(viewer).getByRole('toolbar', { name: 'Transport' })).toBeInTheDocument();
    // jsdom has no WebCodecs: the viewer explains it instead of loading the engine.
    expect(await within(viewer).findByText('Aperçu indisponible')).toBeInTheDocument();
    const timeline = screen.getByRole('region', { name: 'Timeline' });
    expect(within(timeline).getByRole('group', { name: 'Piste track-v1' })).toBeInTheDocument();
    expect(within(timeline).getByRole('option', { name: 'item-1' })).toBeInTheDocument();

    const side = screen.getByRole('region', { name: 'Inspecteur, animations et IA' });
    expect(within(side).getByText('Aucune sélection')).toBeInTheDocument();
    // Keyboard, not a click: in jsdom every element measures 0 px, so the panel library takes any
    // press for a resize grab on a panel edge and swallows it.
    within(side).getByRole('tab', { name: 'Inspecteur' }).focus();
    await user.keyboard('{ArrowRight}');
    expect(within(side).getByRole('tab', { name: 'Animations' })).toHaveAttribute(
      'aria-selected',
      'true',
    );
    await user.keyboard('{ArrowRight}');
    expect(within(side).getByText('Recherche de l’assistant IA…')).toBeInTheDocument();

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
    expect(statusBar).toHaveTextContent('Vérification de l’IA…');
    expect(statusBar).toHaveTextContent('Version 0.0.0-test');
  });

  it('opens the welcome wizard on the first launch and remembers it was skipped', async () => {
    const user = userEvent.setup();
    const preferences = memoryStorage();
    renderApp('/', { preferences });
    const wizard = await screen.findByRole('dialog', { name: 'Bienvenue dans SimpleEffect' });
    expect(wizard).toHaveTextContent('Étape 1 sur 5');
    await user.click(within(wizard).getByRole('button', { name: 'Suivant' }));
    expect(screen.getByRole('dialog', { name: 'Assistant IA' })).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: 'Passer' }));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(preferences.entries.has(ONBOARDING_DONE_KEY)).toBe(true);
  });

  it('opens « À propos » from the version, with its versions, licences and the wizard', async () => {
    const user = userEvent.setup();
    renderApp('/');
    await user.click(screen.getByRole('button', { name: 'Version 0.0.0-test' }));
    const about = screen.getByRole('dialog', { name: 'À propos de SimpleEffect' });
    const versions = within(about).getByRole('region', { name: 'Versions' });
    expect(versions).toHaveTextContent('SimpleEffect0.0.0-test');
    expect(versions).toHaveTextContent('Moteur de rendu0.0.0-engine');
    expect(versions).toHaveTextContent('Electroninconnue');
    await user.click(within(about).getByRole('tab', { name: 'Licences' }));
    expect(await within(about).findByText(/générées à la construction/)).toBeInTheDocument();
    await user.click(within(about).getByRole('tab', { name: 'Application' }));
    await user.click(within(about).getByRole('button', { name: 'Assistant de bienvenue' }));
    expect(
      await screen.findByRole('dialog', { name: 'Bienvenue dans SimpleEffect' }),
    ).toBeInTheDocument();
  });
});

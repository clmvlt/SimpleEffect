/**
 * @vitest-environment jsdom
 */
import { CoreClient, type Asset, type Flicks } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { fireEvent, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { CoreClientProvider } from '../../../shared/core';
import { DesktopBridgeProvider, type SimpleEffectBridge } from '../../../shared/desktop';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { session } from '../../../shared/project-store/testing/session-fixture';
import { I18nextProvider } from 'react-i18next';
import { mediaBinStore } from '../store/media-bin-store';
import { testAsset } from '../testing/assets';
import { MediaBinPanel } from './MediaBinPanel';

const clients: CoreClient[] = [];

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  mediaBinStore.setState({ view: 'grid', kind: 'all', query: '', selectedAssetId: null });
});

const clip = testAsset({ id: '0193e7d2-0000-7000-8000-000000000001' });
const song = testAsset({
  id: '0193e7d2-0000-7000-8000-000000000002',
  originalPath: '/music/Été.wav',
  kind: 'audio',
  missing: true,
});

/** Renders the panel over a project holding `assets`; the core never connects. */
function renderPanel(assets: Asset[], bridge: Partial<SimpleEffectBridge> = {}) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  const loaded = session();
  sync.load({ ...loaded, snapshot: { ...loaded.snapshot, assets } });
  const fullBridge: SimpleEffectBridge = {
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
    ...bridge,
  };
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <DesktopBridgeProvider bridge={fullBridge}>
          <ProjectStoreProvider store={store} sync={sync}>
            <TooltipProvider>
              <MediaBinPanel />
            </TooltipProvider>
          </ProjectStoreProvider>
        </DesktopBridgeProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return client;
}

describe('MediaBinPanel', () => {
  it('invites to import when the project has no media', async () => {
    const chooseFiles = vi.fn(() => Promise.resolve([]));
    renderPanel([], { chooseFiles });
    expect(screen.getByText('Aucun média')).toBeInTheDocument();
    const buttons = screen.getAllByRole('button', { name: 'Importer des médias…' });
    // The header's icon button and the empty state's button.
    const [, emptyStateButton] = buttons;
    if (emptyStateButton === undefined) throw new Error('the empty state offers to import');
    await userEvent.click(emptyStateButton);
    expect(chooseFiles).toHaveBeenCalledWith(
      expect.objectContaining({ title: 'Importer des médias', filterName: 'Médias' }),
    );
  });

  it('lists media with their missing state, filters and searches them', async () => {
    renderPanel([clip, song]);
    const list = screen.getByRole('list');
    expect(within(list).getAllByRole('listitem')).toHaveLength(2);
    expect(screen.getByText('Manquant')).toBeInTheDocument();

    await userEvent.click(screen.getByRole('radio', { name: 'Son' }));
    expect(within(list).getAllByRole('listitem')).toHaveLength(1);
    await userEvent.click(screen.getByRole('radio', { name: 'Tous' }));
    await userEvent.type(screen.getByRole('searchbox', { name: 'Rechercher un média' }), 'ete');
    expect(within(screen.getByRole('list')).getByText('Été.wav')).toBeInTheDocument();
    await userEvent.type(screen.getByRole('searchbox'), 'zzz');
    expect(screen.getByText('Aucun résultat')).toBeInTheDocument();
  });

  it('shows the details of a media and offers to relink a missing one', async () => {
    const chooseFiles = vi.fn(() => Promise.resolve([]));
    renderPanel([clip, song], { chooseFiles });
    await userEvent.click(screen.getByRole('button', { name: 'Ouvrir les détails de Été.wav' }));
    const details = screen.getByRole('complementary', { name: 'Détails' });
    expect(within(details).getByText('Introuvable : /music/Été.wav')).toBeInTheDocument();
    expect(within(details).getByText(/AAC LC · 2 canaux · 48 kHz/)).toBeInTheDocument();
    await userEvent.click(within(details).getByRole('button', { name: 'Relier…' }));
    expect(chooseFiles).toHaveBeenCalledWith(
      expect.objectContaining({ title: 'Relier « Été.wav »' }),
    );
  });

  it('shows how a media plays, its derived copies and its analyses, and retries a failure', async () => {
    const prores = testAsset({
      id: '0193e7d2-0000-7000-8000-000000000003',
      originalPath: '/rushes/interview.mov',
      decodability: { video: 'undecodable', audio: 'decodable' },
      derived: {
        mezzanine: {
          relativePath: 'cache/media/k/mezzanine.mp4',
          status: { kind: 'failed', message: 'disque plein' },
        },
        proxy: { relativePath: 'cache/media/k/proxy-540.mp4', status: { kind: 'ready' } },
      },
      analysis: {
        audio: {
          integratedLufs: -16.5,
          loudnessRangeLu: 5,
          truePeakDbtp: -1,
          silenceCount: 2,
          silenceDuration: (2.25 * 705_600_000) as Flicks,
        },
        transcript: null,
        scenes: { changeCount: 4 },
      },
    });
    const client = renderPanel([prores]);
    const send = vi.spyOn(client, 'send').mockResolvedValue({ kind: 'generateDerivedMedia' });
    expect(screen.getByText('Illisible')).toBeInTheDocument();
    await userEvent.click(
      screen.getByRole('button', { name: 'Ouvrir les détails de interview.mov' }),
    );
    const details = screen.getByRole('complementary', { name: 'Détails' });
    expect(within(details).getByText('Échec : disque plein')).toBeInTheDocument();
    expect(within(details).getByText('Prêt')).toBeInTheDocument();
    expect(within(details).getByText('-16,5 LUFS')).toBeInTheDocument();
    expect(within(details).getByText('2 silences (2,3 s au total)')).toBeInTheDocument();
    expect(within(details).getByText('4 changements')).toBeInTheDocument();
    await userEvent.click(within(details).getByRole('button', { name: 'Réessayer' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'generateDerivedMedia',
      projectId: 'project-1',
      assetId: prores.id,
      derived: 'mezzanine',
    });
  });

  it('imports files dropped from the OS by their paths', () => {
    const client = renderPanel([clip], { getPathForFile: (file) => `/drop/${file.name}` });
    const send = vi
      .spyOn(client, 'send')
      .mockResolvedValue({ kind: 'importMedia', imported: [], duplicates: [], failures: [] });
    const files = [new File(['x'], 'a.mp4'), new File(['y'], 'b.wav')];
    const dataTransfer = { types: ['Files'], files, dropEffect: 'none' };
    const zone = screen.getByRole('list').closest('section');
    if (zone === null) throw new Error('the panel is a section');
    fireEvent.drop(within(zone).getByRole('list'), { dataTransfer });
    expect(send).toHaveBeenCalledWith({
      kind: 'importMedia',
      projectId: 'project-1',
      paths: ['/drop/a.mp4', '/drop/b.wav'],
    });
  });
});

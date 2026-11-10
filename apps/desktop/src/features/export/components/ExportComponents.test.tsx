/**
 * @vitest-environment jsdom
 */
import {
  CoreClient,
  type Command,
  type ExportInfo,
  type Flicks,
  type StartExportRequest,
} from '@se/protocol';
import { Toaster, TooltipProvider } from '@se/ui';
import { act, render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { CoreClientProvider } from '../../../shared/core';
import { DesktopBridgeProvider, type SimpleEffectBridge } from '../../../shared/desktop';
import { resetFontCatalog } from '../../../shared/fonts';
import {
  connectProjectSync,
  createProjectStore,
  ProjectStoreProvider,
} from '../../../shared/project-store';
import { reframeSession } from '../../../shared/project-store/testing/reframe-fixture';
import { ShortcutProvider, ShortcutRegistry } from '../../../shared/shortcuts';
import type { ExportFormDeps } from '../hooks/use-export-form';
import type { EncoderSupport } from '../services/export-warnings';
import { resetExports, setExports } from '../store/exports-store';
import { ExportButton } from './ExportButton';
import { ExportsButton } from './ExportsButton';

const clients: CoreClient[] = [];

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  resetExports();
  resetFontCatalog();
});

function bridge(existing: readonly string[] = []) {
  return {
    getCoreConnection: () => new Promise(() => undefined),
    chooseFolder: vi.fn(() => Promise.resolve('/films')),
    chooseFiles: () => Promise.resolve([]),
    chooseFile: () => Promise.resolve(null),
    chooseSaveFile: () => Promise.resolve(null),
    openExternal: () => Promise.resolve(),
    showInFolder: vi.fn(() => Promise.resolve()),
    getPathForFile: () => '',
    showNotification: () => Promise.resolve(),
    resolveFilePath: vi.fn(({ folder, fileName }: { folder: string; fileName: string }) => {
      const path = `${folder}/${fileName}`;
      return Promise.resolve({ path, exists: existing.includes(path) });
    }),
    setExportActivity: () => Promise.resolve(),
    onNotificationClick: () => () => undefined,
  } satisfies SimpleEffectBridge;
}

function deps(support: EncoderSupport): ExportFormDeps {
  return {
    probe: () => Promise.resolve(support),
    now: () => new Date(2026, 10, 10, 9, 5),
    newId: () => 'preset-1',
  };
}

const READY: EncoderSupport = { video: 'hardware', audio: 'webCodecs' };

function answer(command: Command): unknown {
  switch (command.kind) {
    case 'getExportPreferences':
      return {
        kind: command.kind,
        directory: '/exports',
        fileNameTemplate: '{sequence}',
        presets: [],
      };
    case 'listFonts':
      return { kind: command.kind, families: [{ name: 'Inter', source: 'bundled', faces: [] }] };
    case 'startExport':
      return { kind: command.kind, export: exportInfo('export-1', { kind: 'queued' }) };
    default:
      return { kind: command.kind };
  }
}

function renderExport(ui: React.ReactNode, desktop = bridge()) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command) => Promise.resolve(answer(command) as never));
  const store = createProjectStore();
  const sync = connectProjectSync(client, store);
  sync.load(reframeSession());
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <DesktopBridgeProvider bridge={desktop}>
          <ProjectStoreProvider store={store} sync={sync}>
            <ShortcutProvider registry={new ShortcutRegistry('other')}>
              <TooltipProvider>
                {ui}
                <Toaster label="Notifications" />
              </TooltipProvider>
            </ShortcutProvider>
          </ProjectStoreProvider>
        </DesktopBridgeProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return { send, desktop };
}

const sent = <K extends Command['kind']>(
  send: { mock: { calls: readonly (readonly [Command, ...unknown[]])[] } },
  kind: K,
) =>
  send.mock.calls
    .map(([command]) => command)
    .filter((command): command is Extract<Command, { kind: K }> => command.kind === kind);

async function openDialog(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole('button', { name: 'Exporter' }));
  const dialog = await screen.findByRole('dialog', { name: 'Exporter « Film »' });
  await waitFor(() => {
    expect(within(dialog).getByRole('button', { name: 'Exporter' })).toBeEnabled();
  });
  return dialog;
}

describe('the export dialog', () => {
  it('starts the default preset in the remembered folder and remembers the destination', async () => {
    const user = userEvent.setup();
    const { send } = renderExport(<ExportButton deps={deps(READY)} />);
    const dialog = await openDialog(user);
    expect(within(dialog).getByText(/H\.264 · 12 Mb\/s · AAC 192 kb\/s/)).toBeInTheDocument();
    expect(within(dialog).getByText(/Donne « Film\.mp4 »/)).toBeInTheDocument();
    await user.click(within(dialog).getByRole('button', { name: 'Exporter' }));
    await waitFor(() => {
      expect(sent(send, 'startExport')).toHaveLength(1);
    });
    const request: StartExportRequest | undefined = sent(send, 'startExport')[0];
    expect(request).toMatchObject({
      path: '/exports/Film.mp4',
      presetName: 'YouTube 1080p (H.264)',
      settings: {
        container: 'mp4',
        video: { codec: 'h264', width: 1920, height: 1080 },
        audio: { codec: 'aac', bitsPerSecond: 192_000, encoder: 'webCodecs' },
      },
    });
    expect(request?.range).toBeUndefined();
    expect(sent(send, 'updateExportPreferences')[0]).toMatchObject({
      directory: '/exports',
      fileNameTemplate: '{sequence}',
    });
    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    });
  });

  it('asks before replacing an existing file', async () => {
    const user = userEvent.setup();
    const { send } = renderExport(
      <ExportButton deps={deps(READY)} />,
      bridge(['/exports/Film.mp4']),
    );
    const dialog = await openDialog(user);
    await user.click(within(dialog).getByRole('button', { name: 'Exporter' }));
    const question = await screen.findByRole('alertdialog', { name: 'Remplacer le fichier ?' });
    expect(sent(send, 'startExport')).toHaveLength(0);
    await user.click(within(question).getByRole('button', { name: 'Remplacer' }));
    await waitFor(() => {
      expect(sent(send, 'startExport')[0]?.path).toBe('/exports/Film.mp4');
    });
  });

  it('hands AAC to ffmpeg when this machine cannot encode it', async () => {
    const user = userEvent.setup();
    const { send } = renderExport(
      <ExportButton deps={deps({ video: 'software', audio: 'none' })} />,
    );
    const dialog = await openDialog(user);
    expect(within(dialog).getByText(/encodé par ffmpeg/)).toBeInTheDocument();
    expect(within(dialog).getByText(/l’export sera plus lent/)).toBeInTheDocument();
    await user.click(within(dialog).getByRole('button', { name: 'Exporter' }));
    await waitFor(() => {
      expect(sent(send, 'startExport')[0]?.settings.audio?.encoder).toBe('ffmpeg');
    });
  });

  it('blocks codecs this machine cannot encode', async () => {
    const user = userEvent.setup();
    renderExport(<ExportButton deps={deps({ video: 'none', audio: 'webCodecs' })} />);
    await user.click(screen.getByRole('button', { name: 'Exporter' }));
    const dialog = await screen.findByRole('dialog');
    expect(await within(dialog).findByRole('alert')).toHaveTextContent(/ne sait pas encoder/);
    expect(within(dialog).getByRole('button', { name: 'Exporter' })).toBeDisabled();
  });

  it('opens with Ctrl+M', async () => {
    const user = userEvent.setup();
    renderExport(<ExportButton deps={deps(READY)} />);
    await user.keyboard('{Control>}m{/Control}');
    expect(await screen.findByRole('dialog', { name: 'Exporter « Film »' })).toBeInTheDocument();
  });
});

function exportInfo(id: string, state: ExportInfo['state']): ExportInfo {
  return {
    id,
    jobId: null,
    projectId: 'project-1',
    sequenceId: 'sequence-1',
    sequenceName: 'Film',
    presetName: 'YouTube 1080p (H.264)',
    path: `/exports/${id}.mp4`,
    settings: { container: 'wav', video: null, audio: null },
    duration: 1000 as Flicks,
    createdAt: 0,
    hasThumbnail: false,
    state,
  };
}

describe('the exports menu', () => {
  it('is hidden without exports', () => {
    renderExport(<ExportsButton />);
    expect(screen.queryByRole('button', { name: 'Exports' })).not.toBeInTheDocument();
  });

  it('follows progress, cancels, reveals and clears the finished exports', async () => {
    const user = userEvent.setup();
    const { send, desktop } = renderExport(<ExportsButton />);
    act(() => {
      setExports([
        exportInfo('done', {
          kind: 'completed',
          sizeBytes: 2_500_000,
          elapsedSeconds: 75,
          check: null,
        }),
        exportInfo('running', { kind: 'rendering', progress: 0.42, etaSeconds: 90 }),
      ]);
    });
    const pill = screen.getByRole('button', { name: 'Exports' });
    expect(pill).toHaveTextContent('Export 42 %');
    await user.click(pill);
    const running = await screen.findByRole('listitem', { name: 'running.mp4' });
    expect(within(running).getByText('42 % · 1:30 restantes')).toBeInTheDocument();
    expect(within(running).getByRole('progressbar')).toBeInTheDocument();
    await user.click(within(running).getByRole('button', { name: /Annuler l’export/ }));
    await waitFor(() => {
      expect(sent(send, 'cancelExport')[0]).toMatchObject({ exportId: 'running' });
    });
    const done = screen.getByRole('listitem', { name: 'done.mp4' });
    expect(within(done).getByText(/rendu en 1:15/)).toBeInTheDocument();
    await user.click(within(done).getByRole('button', { name: /Afficher done\.mp4/ }));
    expect(desktop.showInFolder).toHaveBeenCalledWith('/exports/done.mp4');
    await user.click(screen.getByRole('button', { name: 'Effacer les terminés' }));
    await waitFor(() => {
      expect(sent(send, 'clearFinishedExports')).toHaveLength(1);
    });
  });
});

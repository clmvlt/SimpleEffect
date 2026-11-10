/**
 * @vitest-environment jsdom
 */
import {
  CoreClient,
  type AiSettings,
  type AiStatus,
  type Command,
  type MediaSettings,
} from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { CoreClientProvider } from '../../../shared/core';
import { DesktopBridgeProvider, type SimpleEffectBridge } from '../../../shared/desktop';
import { settingsDialogStore } from '../store/settings-dialog-store';
import { SettingsButton } from './SettingsButton';

const clients: CoreClient[] = [];

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  settingsDialogStore.setState({ open: false, section: 'media' });
});

const AI_SETTINGS: AiSettings = {
  provider: 'claude-code',
  executablePath: null,
  defaultModel: null,
  defaultEffort: null,
  maxConcurrentTasks: 3,
};

const AI_STATUS: AiStatus = {
  provider: {
    id: 'claude-code',
    name: 'Claude Code',
    command: 'claude',
    loginCommand: '/login',
    helpUrl: 'https://code.claude.com/docs/en/setup',
  },
  installation: { kind: 'installed', path: '/usr/local/bin/claude', version: '2.1.317' },
  auth: { kind: 'loggedOut' },
  capabilities: {
    resumeSessions: true,
    imageInput: true,
    models: [{ id: 'opus', label: 'Opus' }],
    effortLevels: ['low', 'high'],
  },
};

const SETTINGS: MediaSettings = {
  mezzanineQuality: 14,
  proxyResolution: '540p',
  autoProxies: true,
  toolsDir: null,
};

/** Answers the settings commands like a core with an empty 2 Mo cache. */
function answer(command: Command): unknown {
  switch (command.kind) {
    case 'getMediaSettings':
      return { kind: command.kind, settings: SETTINGS };
    case 'updateMediaSettings': {
      const { kind, ...settings } = command;
      return { kind, settings };
    }
    case 'getMediaToolsStatus':
      return {
        kind: command.kind,
        status: {
          ffmpegPath: 'C:\\SimpleEffect\\ffmpeg.exe',
          ffprobePath: 'C:\\SimpleEffect\\ffprobe.exe',
          ffmpegVersion: '9.0.2',
          hardwareEncoders: ['nvenc'],
          problem: null,
        },
      };
    case 'getAiSettings':
      return { kind: command.kind, settings: AI_SETTINGS };
    case 'updateAiSettings': {
      const { kind, ...settings } = command;
      return { kind, settings };
    }
    case 'refreshAiStatus':
    case 'getAiStatus':
      return { kind: command.kind, status: AI_STATUS };
    case 'getProjectCache':
      return {
        kind: command.kind,
        cache: { path: 'C:\\Films\\Film.seproj\\cache', sizeBytes: 2_097_152 },
      };
    case 'clearProjectCache':
      return { kind: command.kind, cache: { path: 'C:\\Films\\Film.seproj\\cache', sizeBytes: 0 } };
    default:
      throw new Error(`unexpected command ${command.kind}`);
  }
}

function renderButton(projectId: string | null, bridge: Partial<SimpleEffectBridge> = {}) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command) => Promise.resolve(answer(command) as never));
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
          <TooltipProvider>
            <SettingsButton projectId={projectId} />
          </TooltipProvider>
        </DesktopBridgeProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return send;
}

describe('SettingsDialog', () => {
  it('shows the media settings and the tools, and saves changes as they are made', async () => {
    const chooseFolder = vi.fn(() => Promise.resolve('D:\\ffmpeg'));
    const send = renderButton(null, { chooseFolder });
    await userEvent.click(screen.getByRole('button', { name: 'Réglages' }));
    const dialog = await screen.findByRole('dialog', { name: 'Réglages' });
    expect(await within(dialog).findByText('CRF 14')).toBeInTheDocument();
    expect(await within(dialog).findByText('NVENC')).toBeInTheDocument();
    expect(within(dialog).getByText('Ouvrez un projet pour voir son cache.')).toBeInTheDocument();

    await userEvent.click(within(dialog).getByRole('radio', { name: '720p' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'updateMediaSettings',
      ...SETTINGS,
      proxyResolution: '720p',
    });
    await userEvent.click(
      within(dialog).getByRole('switch', { name: 'Générer les proxies automatiquement' }),
    );
    expect(send).toHaveBeenLastCalledWith(
      expect.objectContaining({ kind: 'updateMediaSettings', autoProxies: false }),
    );
    await userEvent.click(within(dialog).getByRole('button', { name: 'Choisir…' }));
    expect(send).toHaveBeenCalledWith(
      expect.objectContaining({ kind: 'updateMediaSettings', toolsDir: 'D:\\ffmpeg' }),
    );
    // A new tools folder is checked again.
    await vi.waitFor(() => {
      expect(send).toHaveBeenLastCalledWith({ kind: 'getMediaToolsStatus' });
    });
  });

  it('shows the project cache and cleans it after confirmation', async () => {
    const send = renderButton('project-1');
    await userEvent.click(screen.getByRole('button', { name: 'Réglages' }));
    const dialog = await screen.findByRole('dialog', { name: 'Réglages' });
    expect(await within(dialog).findByText('2 Mo')).toBeInTheDocument();
    await userEvent.click(
      within(dialog).getByRole('button', { name: 'Nettoyer le cache du projet' }),
    );
    const confirm = await screen.findByRole('alertdialog', {
      name: 'Nettoyer le cache du projet ?',
    });
    await userEvent.click(within(confirm).getByRole('button', { name: 'Nettoyer' }));
    expect(send).toHaveBeenCalledWith({ kind: 'clearProjectCache', projectId: 'project-1' });
    expect(await within(dialog).findByText('0 o')).toBeInTheDocument();
  });

  it('shows the AI provider, refreshes it and saves the AI settings', async () => {
    const chooseFile = vi.fn(() => Promise.resolve('/opt/claude/bin/claude'));
    const send = renderButton(null, { chooseFile });
    await userEvent.click(screen.getByRole('button', { name: 'Réglages' }));
    const dialog = await screen.findByRole('dialog', { name: 'Réglages' });
    await userEvent.click(within(dialog).getByRole('tab', { name: 'IA' }));
    expect(send).toHaveBeenCalledWith({ kind: 'getAiSettings' });
    await userEvent.click(await within(dialog).findByRole('button', { name: 'Actualiser' }));
    expect(send).toHaveBeenCalledWith({ kind: 'refreshAiStatus' });
    expect(
      await within(dialog).findByText('Claude Code 2.1.317 — /usr/local/bin/claude'),
    ).toBeInTheDocument();
    expect(
      within(dialog).getByText(
        'Déconnecté : ouvre un terminal, lance `claude`, puis tape `/login`.',
      ),
    ).toBeInTheDocument();

    await userEvent.click(within(dialog).getByRole('button', { name: 'Choisir…' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'updateAiSettings',
      ...AI_SETTINGS,
      executablePath: '/opt/claude/bin/claude',
    });
    within(dialog).getByRole('slider', { name: 'Tâches simultanées' }).focus();
    await userEvent.keyboard('{ArrowRight}');
    await vi.waitFor(() => {
      expect(send).toHaveBeenLastCalledWith(
        expect.objectContaining({ kind: 'updateAiSettings', maxConcurrentTasks: 4 }),
      );
    });
  });

  it('opens on the section asked by other parts of the app', async () => {
    renderButton(null);
    settingsDialogStore.setState({ open: true, section: 'ai' });
    const dialog = await screen.findByRole('dialog', { name: 'Réglages' });
    expect(within(dialog).getByRole('tab', { name: 'IA' })).toHaveAttribute(
      'aria-selected',
      'true',
    );
  });
});

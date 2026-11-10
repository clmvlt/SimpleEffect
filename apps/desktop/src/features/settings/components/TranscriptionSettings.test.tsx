/**
 * @vitest-environment jsdom
 */
import { CoreClient, type Command, type TranscriptionStatus } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { act, render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { CoreClientProvider } from '../../../shared/core';
import { DesktopBridgeProvider, type SimpleEffectBridge } from '../../../shared/desktop';
import { openSettings, settingsDialogStore } from '../store/settings-dialog-store';
import { testModel, transcriptionStatus } from '../testing/transcription-fixture';
import { SettingsButton } from './SettingsButton';

const clients: CoreClient[] = [];

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  settingsDialogStore.setState({ open: false, section: 'media' });
});

const MEGABYTE = 1_048_576;
const TURBO = { kind: 'model', modelId: 'large-v3-turbo' } as const;
const IMPORTED = testModel('ggml-fr.bin', {
  fileName: 'ggml-fr.bin',
  custom: true,
  installed: true,
  size: 500 * MEGABYTE,
});

/** Answers like a core holding `status` (the other sections get minimal answers). */
function answer(command: Command, status: TranscriptionStatus): unknown {
  switch (command.kind) {
    case 'getTranscriptionStatus':
    case 'removeTranscriptionFile':
      return { kind: command.kind, status };
    case 'updateTranscriptionPreferences': {
      const { kind, ...preferences } = command;
      return { kind, status: { ...status, preferences } };
    }
    case 'startTranscriptionDownload':
    case 'cancelTranscriptionDownload':
      return { kind: command.kind };
    case 'importTranscriptionModel':
      return {
        kind: command.kind,
        modelId: IMPORTED.id,
        status: { ...status, models: [...status.models, IMPORTED] },
      };
    case 'getMediaSettings':
      return {
        kind: command.kind,
        settings: {
          mezzanineQuality: 14,
          proxyResolution: '540p',
          autoProxies: true,
          toolsDir: null,
        },
      };
    case 'getMediaToolsStatus':
      return {
        kind: command.kind,
        status: {
          ffmpegPath: null,
          ffprobePath: null,
          ffmpegVersion: null,
          hardwareEncoders: [],
          problem: null,
        },
      };
    case 'getAiSettings':
      return {
        kind: command.kind,
        settings: {
          provider: 'claude-code',
          executablePath: null,
          defaultModel: null,
          defaultEffort: null,
          maxConcurrentTasks: 3,
        },
      };
    default:
      throw new Error(`unexpected command ${command.kind}`);
  }
}

/** Opens « Réglages → Transcription » over a core holding `status`. */
async function openTranscription(
  status: TranscriptionStatus = transcriptionStatus(),
  bridge: Partial<SimpleEffectBridge> = {},
) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const on = vi.spyOn(client, 'on');
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command) => Promise.resolve(answer(command, status) as never));
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
            <SettingsButton projectId={null} />
          </TooltipProvider>
        </DesktopBridgeProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  act(() => {
    openSettings('transcription');
  });
  const dialog = await screen.findByRole('dialog', { name: 'Réglages' });
  await within(dialog).findByRole('heading', { name: 'Modèles' });
  /** Delivers a `transcriptionStatusChanged` event to the dialog. */
  const emit = (next: TranscriptionStatus) => {
    const call = on.mock.calls.findLast(([kind]) => kind === 'transcriptionStatusChanged');
    act(() => {
      call?.[1]({ kind: 'transcriptionStatusChanged', status: next });
    });
  };
  return { dialog, send, emit };
}

/** A copy of `status` with the download of `large-v3-turbo` set to `download`. */
function withTurboDownload(
  status: TranscriptionStatus,
  download: TranscriptionStatus['models'][number]['download'],
): TranscriptionStatus {
  return {
    ...status,
    models: status.models.map((model) =>
      model.id === TURBO.modelId ? { ...model, download } : model,
    ),
  };
}

describe('Réglages → Transcription', () => {
  it('shows whisper.cpp, the GPU and the models with the recommended one', async () => {
    const { dialog, send } = await openTranscription();
    expect(send).toHaveBeenCalledWith({ kind: 'getTranscriptionStatus' });
    expect(within(dialog).getByRole('tab', { name: 'Transcription' })).toHaveAttribute(
      'aria-selected',
      'true',
    );
    expect(
      within(dialog).getByText('whisper.cpp — C:\\SimpleEffect\\whisper-cli.exe'),
    ).toBeInTheDocument();
    expect(within(dialog).getByText('NVIDIA GeForce RTX 3080')).toBeInTheDocument();
    expect(within(dialog).getByText('Processeur')).toBeInTheDocument();
    const turbo = within(dialog).getByRole('listitem', { name: 'large-v3-turbo' });
    expect(within(turbo).getByText('Recommandé')).toBeInTheDocument();
    expect(within(turbo).getByText('1,5 Go')).toBeInTheDocument();
    const base = within(dialog).getByRole('listitem', { name: 'base' });
    expect(within(base).queryByText('Recommandé')).not.toBeInTheDocument();
    expect(within(base).getByRole('button', { name: 'Supprimer' })).toBeInTheDocument();
    expect(within(dialog).getByText(/Modèle de voix manquant/)).toBeInTheDocument();
  });

  it('downloads a model only after consent and follows its progress', async () => {
    const status = transcriptionStatus();
    const { dialog, send, emit } = await openTranscription(status);
    const row = () => within(dialog).getByRole('listitem', { name: 'large-v3-turbo' });

    await userEvent.click(within(row()).getByRole('button', { name: 'Télécharger' }));
    let consent = await screen.findByRole('alertdialog', {
      name: 'Télécharger le modèle large-v3-turbo ?',
    });
    expect(within(consent).getByText(/Hugging Face/)).toBeInTheDocument();
    expect(within(consent).getByText(/détection de la voix \(864,3 Ko\)/)).toBeInTheDocument();
    await userEvent.click(within(consent).getByRole('button', { name: 'Annuler' }));
    expect(send).not.toHaveBeenCalledWith(
      expect.objectContaining({ kind: 'startTranscriptionDownload' }),
    );

    await userEvent.click(within(row()).getByRole('button', { name: 'Télécharger' }));
    consent = await screen.findByRole('alertdialog');
    await userEvent.click(within(consent).getByRole('button', { name: 'Télécharger' }));
    expect(send).toHaveBeenCalledWith({ kind: 'startTranscriptionDownload', target: TURBO });

    const half = (1549 * MEGABYTE) / 2;
    emit(withTurboDownload(status, { kind: 'downloading', received: half, total: 2 * half }));
    expect(
      within(row()).getByRole('progressbar', { name: 'Téléchargement de large-v3-turbo' }),
    ).toHaveAttribute('aria-valuenow', '50');
    expect(within(row()).getByText('774,5 Mo / 1,5 Go')).toBeInTheDocument();
    await userEvent.click(within(row()).getByRole('button', { name: 'Annuler' }));
    expect(send).toHaveBeenCalledWith({ kind: 'cancelTranscriptionDownload', target: TURBO });

    emit(withTurboDownload(status, { kind: 'verifying' }));
    expect(within(row()).getAllByText('Vérification…')[0]).toBeInTheDocument();
    emit(withTurboDownload(status, { kind: 'failed', message: 'SHA-256 différent' }));
    expect(within(row()).getByRole('alert')).toHaveTextContent('SHA-256 différent');
    send.mockClear();
    await userEvent.click(within(row()).getByRole('button', { name: 'Réessayer' }));
    expect(send).toHaveBeenCalledWith({ kind: 'startTranscriptionDownload', target: TURBO });
  });

  it('saves the model used and the options as they change', async () => {
    const status = transcriptionStatus();
    const { dialog, send } = await openTranscription(status);
    expect(within(dialog).getByRole('radio', { name: 'Automatique (recommandé)' })).toBeChecked();
    expect(within(dialog).queryByRole('radio', { name: 'tiny' })).not.toBeInTheDocument();

    await userEvent.click(within(dialog).getByRole('radio', { name: 'base' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'updateTranscriptionPreferences',
      ...status.preferences,
      model: 'base',
    });
    expect(within(dialog).getByRole('radio', { name: 'base' })).toBeChecked();

    await userEvent.click(within(dialog).getByRole('switch', { name: 'Utiliser le GPU' }));
    expect(send).toHaveBeenLastCalledWith(
      expect.objectContaining({ kind: 'updateTranscriptionPreferences', useGpu: false }),
    );
    await userEvent.click(
      within(dialog).getByRole('switch', { name: 'Transcrire les médias à l’import' }),
    );
    expect(send).toHaveBeenLastCalledWith(
      expect.objectContaining({ kind: 'updateTranscriptionPreferences', autoTranscribe: true }),
    );
  });

  it('imports a model chosen with the file picker', async () => {
    const chooseFile = vi.fn(() => Promise.resolve('D:\\Modèles\\ggml-fr.bin'));
    const { dialog, send } = await openTranscription(transcriptionStatus(), { chooseFile });
    await userEvent.click(within(dialog).getByRole('button', { name: 'Importer un modèle…' }));
    expect(chooseFile).toHaveBeenCalledWith({
      title: 'Modèle whisper.cpp (fichier ggml .bin)',
      buttonLabel: 'Importer ce modèle',
    });
    expect(send).toHaveBeenCalledWith({
      kind: 'importTranscriptionModel',
      path: 'D:\\Modèles\\ggml-fr.bin',
    });
    const row = await within(dialog).findByRole('listitem', { name: 'ggml-fr.bin' });
    expect(within(row).getByText('Importé')).toBeInTheDocument();
    expect(within(dialog).getByRole('radio', { name: 'ggml-fr.bin' })).toBeInTheDocument();
  });

  it('offers the GPU pack after consent, and only on platforms that have one', async () => {
    const { dialog, send } = await openTranscription();
    await userEvent.click(
      within(dialog).getByRole('button', { name: 'Installer le pack GPU (685 Mo)' }),
    );
    const consent = await screen.findByRole('alertdialog', { name: 'Installer le pack GPU ?' });
    expect(
      within(consent).getByText(/version CUDA officielle.*GitHub.*SHA-256/),
    ).toBeInTheDocument();
    await userEvent.click(within(consent).getByRole('button', { name: 'Télécharger' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'startTranscriptionDownload',
      target: { kind: 'gpuRuntime' },
    });
    expect(
      within(dialog).getByRole('progressbar', { name: 'Téléchargement de pack GPU' }),
    ).toBeInTheDocument();
  });

  it('removes an installed GPU pack after confirmation', async () => {
    const base = transcriptionStatus();
    const status = transcriptionStatus({
      backend: 'cuda',
      lastBackend: 'cuda',
      gpuRuntime: base.gpuRuntime === null ? null : { ...base.gpuRuntime, installed: true },
    });
    const { dialog, send } = await openTranscription(status);
    expect(within(dialog).getAllByText('GPU (CUDA)')).toHaveLength(2);
    const pack = within(dialog).getByRole('region', { name: 'Pack GPU' });
    expect(within(pack).getByText('Installé')).toBeInTheDocument();
    await userEvent.click(within(pack).getByRole('button', { name: 'Supprimer' }));
    const confirm = await screen.findByRole('alertdialog', { name: 'Supprimer le pack GPU ?' });
    await userEvent.click(within(confirm).getByRole('button', { name: 'Supprimer' }));
    expect(send).toHaveBeenCalledWith({
      kind: 'removeTranscriptionFile',
      target: { kind: 'gpuRuntime' },
    });
  });

  it('hides the GPU pack where none exists and explains a missing whisper.cpp', async () => {
    const { dialog } = await openTranscription(
      transcriptionStatus({
        gpuRuntime: null,
        gpuName: null,
        whisper: { path: null, problem: 'whisper-cli est introuvable dans resources/bin.' },
      }),
    );
    expect(within(dialog).queryByRole('region', { name: 'Pack GPU' })).not.toBeInTheDocument();
    expect(within(dialog).getByText('Aucun GPU compatible détecté')).toBeInTheDocument();
    expect(
      within(dialog).getByText('whisper-cli est introuvable dans resources/bin.'),
    ).toBeInTheDocument();
  });
});

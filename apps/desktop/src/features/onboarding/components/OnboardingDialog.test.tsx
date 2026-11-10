/**
 * @vitest-environment jsdom
 */
import { CoreClient, type Command } from '@se/protocol';
import { TooltipProvider } from '@se/ui';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { I18nextProvider } from 'react-i18next';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { createI18n } from '../../../i18n';
import { CoreClientProvider } from '../../../shared/core';
import { DesktopBridgeProvider, type SimpleEffectBridge } from '../../../shared/desktop';
import type { KeyValueStorage } from '../../../shared/storage';
import { resetVisionStore } from '../../reframing';
import { aiStepState } from '../services/ai-step';
import { codecsCheck, toolsCheck } from '../services/media-step';
import {
  ONBOARDING_DONE_KEY,
  isFirstRun,
  nextStep,
  previousStep,
} from '../services/onboarding-steps';
import { onboardingStore } from '../store/onboarding-store';
import { OnboardingDialog } from './OnboardingDialog';

const clients: CoreClient[] = [];

afterEach(() => {
  for (const client of clients.splice(0)) client.close();
  onboardingStore.setState({ open: false, step: 'welcome' });
  resetVisionStore();
});

function memoryStorage(): KeyValueStorage & { entries: Map<string, string> } {
  const entries = new Map<string, string>();
  return {
    entries,
    getItem: (key) => entries.get(key) ?? null,
    setItem: (key, value) => {
      entries.set(key, value);
    },
  };
}

/** Answers the wizard's commands like a core with ffmpeg, no face detection and no model. */
function answer(command: Command): unknown {
  switch (command.kind) {
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
    case 'getVisionStatus':
      return {
        kind: command.kind,
        status: {
          available: false,
          model: 'yunet-2026may',
          runtime: null,
          accelerator: null,
          reason: 'ONNX Runtime is missing',
        },
      };
    case 'getTranscriptionStatus':
      return {
        kind: command.kind,
        status: {
          whisper: { path: null, problem: null },
          gpuName: null,
          backend: 'cpu',
          lastBackend: null,
          gpuRuntime: null,
          models: [],
          voiceModel: { installed: false, size: 885_000, download: null },
          modelsDir: 'C:\\data\\models',
          preferences: {
            model: null,
            useGpu: true,
            voiceDetection: false,
            transcribeOnImport: false,
            whisperDir: null,
            modelsDir: null,
          },
        },
      };
    case 'getNewProjectDefaults':
      return { kind: command.kind, parentDir: 'C:\\Users\\Marie\\Documents\\SimpleEffect' };
    case 'setProjectsFolder':
      return { kind: command.kind, defaults: { parentDir: command.path } };
    default:
      throw new Error(`unexpected command ${command.kind}`);
  }
}

function renderWizard(preferences: KeyValueStorage) {
  const client = CoreClient.connect(() => new Promise(() => undefined));
  clients.push(client);
  const send = vi
    .spyOn(client, 'send')
    .mockImplementation((command) => Promise.resolve(answer(command) as never));
  const bridge = {
    chooseFolder: vi.fn(() => Promise.resolve('D:\\Montages')),
    openExternal: () => Promise.resolve(),
  } as unknown as SimpleEffectBridge;
  render(
    <I18nextProvider i18n={createI18n()}>
      <CoreClientProvider client={client}>
        <DesktopBridgeProvider bridge={bridge}>
          <TooltipProvider>
            <OnboardingDialog preferences={preferences} />
          </TooltipProvider>
        </DesktopBridgeProvider>
      </CoreClientProvider>
    </I18nextProvider>,
  );
  return { send, bridge };
}

describe('first-run wizard', () => {
  it('walks through every step, saves the projects folder and remembers it is done', async () => {
    const user = userEvent.setup();
    const preferences = memoryStorage();
    const { send } = renderWizard(preferences);
    const next = () => user.click(screen.getByRole('button', { name: 'Suivant' }));

    expect(
      await screen.findByRole('dialog', { name: 'Bienvenue dans SimpleEffect' }),
    ).toBeVisible();
    await next();
    expect(screen.getByRole('dialog', { name: 'Assistant IA' })).toHaveTextContent(
      'Recherche en cours…',
    );
    await next();
    const media = screen.getByRole('dialog', { name: 'Binaires et GPU' });
    expect(
      await within(media).findByText('Version 9.0.2, encodeurs matériels : nvenc.'),
    ).toBeVisible();
    expect(await within(media).findByText('Indisponible : ONNX Runtime is missing')).toBeVisible();
    await next();
    expect(
      screen.getByRole('dialog', { name: 'Modèles de transcription (facultatif)' }),
    ).toBeVisible();
    await next();
    expect(await screen.findByTestId('projects-folder')).toHaveTextContent(
      'C:\\Users\\Marie\\Documents\\SimpleEffect',
    );
    await user.click(screen.getByRole('button', { name: 'Choisir un autre dossier…' }));
    expect(await screen.findByText('D:\\Montages')).toBeVisible();
    expect(send).toHaveBeenCalledWith({ kind: 'setProjectsFolder', path: 'D:\\Montages' });

    await user.click(screen.getByRole('button', { name: 'Précédent' }));
    expect(
      screen.getByRole('dialog', { name: 'Modèles de transcription (facultatif)' }),
    ).toBeVisible();
    await next();
    await user.click(screen.getByRole('button', { name: 'Terminer' }));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(preferences.entries.has(ONBOARDING_DONE_KEY)).toBe(true);
  });

  it('stays closed once done', () => {
    const preferences = memoryStorage();
    preferences.setItem(ONBOARDING_DONE_KEY, '2026-11-01T00:00:00.000Z');
    renderWizard(preferences);
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });
});

describe('wizard services', () => {
  it('orders the steps and knows a first run', () => {
    expect(nextStep('welcome')).toBe('ai');
    expect(nextStep('projects')).toBeNull();
    expect(previousStep('welcome')).toBeNull();
    expect(previousStep('models')).toBe('media');
    expect(isFirstRun(memoryStorage())).toBe(true);
  });

  it('turns statuses into check rows', () => {
    expect(aiStepState(null).installationKey).toBe('checking');
    expect(toolsCheck('failed').tone).toBe('danger');
    expect(codecsCheck([{ codec: 'H.264', decode: true, encode: false }])).toMatchObject({
      tone: 'warning',
      values: { decode: 'H.264', encode: '—' },
    });
  });
});

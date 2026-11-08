import { describe, expect, it } from 'vitest';

import { settingsSectionOf } from '../store/settings-dialog-store';
import { testModel, transcriptionStatus } from '../testing/transcription-fixture';
import {
  downloadPercent,
  downloadPhase,
  withDownloadCleared,
  withDownloadStarted,
} from './transcription-downloads';
import {
  AUTOMATIC_MODEL,
  downloadsVoiceModelToo,
  hasInstalledModel,
  modelChoiceValue,
  modelChoices,
  modelFromChoice,
  modelLabel,
} from './transcription-models';

describe('transcription downloads', () => {
  it('tells where a file stands, a download winning over its installed state', () => {
    expect(downloadPhase({ installed: false, download: null })).toBe('missing');
    expect(downloadPhase({ installed: true, download: null })).toBe('installed');
    expect(downloadPhase({ installed: true, download: { kind: 'verifying' } })).toBe('verifying');
    expect(downloadPhase({ installed: false, download: { kind: 'failed', message: '…' } })).toBe(
      'failed',
    );
  });

  it('computes the percentage of a running download', () => {
    expect(downloadPercent({ kind: 'downloading', received: 25, total: 100 })).toBe(25);
    expect(downloadPercent({ kind: 'downloading', received: 150, total: 100 })).toBe(100);
    expect(downloadPercent({ kind: 'downloading', received: 10, total: 0 })).toBeNull();
    expect(downloadPercent({ kind: 'verifying' })).toBeNull();
  });

  it('shows a started download at 0 of its size, then clears it', () => {
    const status = transcriptionStatus();
    const model = withDownloadStarted(status, { kind: 'model', modelId: 'large-v3-turbo' });
    const turbo = model.models.find((candidate) => candidate.id === 'large-v3-turbo');
    expect(turbo?.download).toEqual({ kind: 'downloading', received: 0, total: turbo?.size });
    expect(model.models.find((candidate) => candidate.id === 'tiny')?.download).toBeNull();
    const cleared = withDownloadCleared(model, { kind: 'model', modelId: 'large-v3-turbo' });
    expect(cleared.models).toEqual(status.models);

    const gpu = withDownloadStarted(status, { kind: 'gpuRuntime' });
    expect(gpu.gpuRuntime?.download).toEqual({
      kind: 'downloading',
      received: 0,
      total: status.gpuRuntime?.downloadSize,
    });
    const voice = withDownloadStarted(status, { kind: 'voiceModel' });
    expect(voice.voiceModel.download).toEqual({ kind: 'downloading', received: 0, total: 885_000 });
  });

  it('leaves a platform without GPU pack unchanged', () => {
    const status = transcriptionStatus({ gpuRuntime: null });
    expect(withDownloadStarted(status, { kind: 'gpuRuntime' })).toBe(status);
  });
});

describe('transcription models', () => {
  it('names imported models by their file name', () => {
    expect(modelLabel(testModel('base'))).toBe('base');
    expect(modelLabel(testModel('fr.bin', { custom: true, fileName: 'ggml-fr.bin' }))).toBe(
      'ggml-fr.bin',
    );
  });

  it('offers the installed models, and keeps a removed choice visible', () => {
    const status = transcriptionStatus();
    expect(modelChoices(status)).toEqual([{ value: 'base', label: 'base' }]);
    expect(modelChoiceValue(status)).toBe(AUTOMATIC_MODEL);
    const removed = transcriptionStatus({
      preferences: { ...status.preferences, model: 'medium' },
    });
    expect(modelChoices(removed)).toEqual([
      { value: 'base', label: 'base' },
      { value: 'medium', label: 'medium' },
    ]);
    expect(modelChoiceValue(removed)).toBe('medium');
  });

  it('maps the automatic choice to no preference', () => {
    expect(modelFromChoice(AUTOMATIC_MODEL)).toBeNull();
    expect(modelFromChoice('small')).toBe('small');
  });

  it('knows whether a model is installed and whether the voice model comes along', () => {
    const status = transcriptionStatus();
    expect(hasInstalledModel(status)).toBe(true);
    expect(hasInstalledModel({ ...status, models: [testModel('tiny')] })).toBe(false);
    expect(downloadsVoiceModelToo(status)).toBe(true);
    const coming = { ...status.voiceModel, download: { kind: 'verifying' as const } };
    expect(downloadsVoiceModelToo({ ...status, voiceModel: coming })).toBe(false);
    const installed = { ...status.voiceModel, installed: true };
    expect(downloadsVoiceModelToo({ ...status, voiceModel: installed })).toBe(false);
  });
});

describe('settingsSectionOf', () => {
  it('reads tab values and falls back to « Médias »', () => {
    expect(settingsSectionOf('transcription')).toBe('transcription');
    expect(settingsSectionOf('ai')).toBe('ai');
    expect(settingsSectionOf('other')).toBe('media');
  });
});

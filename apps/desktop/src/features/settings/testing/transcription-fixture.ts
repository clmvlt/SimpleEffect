import type { TranscriptionModelInfo, TranscriptionStatus } from '@se/protocol';

const MEGABYTE = 1_048_576;

/** A catalogue model, not installed unless `overrides` say so. */
export function testModel(
  id: string,
  overrides: Partial<TranscriptionModelInfo> = {},
): TranscriptionModelInfo {
  return {
    id,
    fileName: `ggml-${id}.bin`,
    size: 100 * MEGABYTE,
    installed: false,
    custom: false,
    recommended: false,
    download: null,
    ...overrides,
  };
}

/**
 * The status of a Windows machine with an RTX 3080, whisper.cpp found, the GPU pack offered and
 * not installed, `base` installed, `large-v3-turbo` recommended, and no voice model.
 */
export function transcriptionStatus(
  overrides: Partial<TranscriptionStatus> = {},
): TranscriptionStatus {
  return {
    whisper: { path: 'C:\\SimpleEffect\\whisper-cli.exe', problem: null },
    gpuName: 'NVIDIA GeForce RTX 3080',
    backend: 'cpu',
    lastBackend: null,
    gpuRuntime: {
      name: 'CUDA 12.4',
      downloadSize: 685 * MEGABYTE,
      installed: false,
      download: null,
    },
    models: [
      testModel('tiny', { size: 75 * MEGABYTE }),
      testModel('base', { size: 142 * MEGABYTE, installed: true }),
      testModel('large-v3-turbo', { size: 1549 * MEGABYTE, recommended: true }),
    ],
    voiceModel: { size: 885_000, installed: false, download: null },
    modelsDir: 'C:\\Users\\me\\AppData\\Roaming\\SimpleEffect\\models\\whisper',
    preferences: {
      model: null,
      useGpu: true,
      autoTranscribe: false,
      voiceDetection: true,
      whisperDir: null,
    },
    ...overrides,
  };
}

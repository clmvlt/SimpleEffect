import type { DownloadProgress, DownloadTarget, TranscriptionStatus } from '@se/protocol';

/** Something « Réglages → Transcription » can download: a model, the voice model, the GPU pack. */
export interface Downloadable {
  readonly installed: boolean;
  /** Its download, while one runs or after one failed. */
  readonly download: DownloadProgress | null;
}

/** Where a downloadable file stands, which decides what its row shows. */
export type DownloadPhase = 'missing' | 'installed' | DownloadProgress['kind'];

/** The phase of `file`: a running or failed download wins over its installed state. */
export function downloadPhase(file: Downloadable): DownloadPhase {
  if (file.download !== null) return file.download.kind;
  return file.installed ? 'installed' : 'missing';
}

/** Completion of a running download in percent (0–100), or `null` when its size is unknown. */
export function downloadPercent(progress: DownloadProgress): number | null {
  if (progress.kind !== 'downloading' || progress.total <= 0) return null;
  return Math.min(100, Math.max(0, (progress.received / progress.total) * 100));
}

/** `status` with the download of `target` replaced by `download(size of the target)`. */
function withTargetDownload(
  status: TranscriptionStatus,
  target: DownloadTarget,
  download: (size: number) => DownloadProgress | null,
): TranscriptionStatus {
  switch (target.kind) {
    case 'model':
      return {
        ...status,
        models: status.models.map((model) =>
          model.id === target.modelId ? { ...model, download: download(model.size) } : model,
        ),
      };
    case 'voiceModel':
      return {
        ...status,
        voiceModel: { ...status.voiceModel, download: download(status.voiceModel.size) },
      };
    case 'gpuRuntime':
      return status.gpuRuntime === null
        ? status
        : {
            ...status,
            gpuRuntime: {
              ...status.gpuRuntime,
              download: download(status.gpuRuntime.downloadSize),
            },
          };
  }
}

/**
 * `status` showing the download of `target` as just started (0 of its size), so its row reacts at
 * once; the core's `transcriptionStatusChanged` events then replace it.
 */
export function withDownloadStarted(
  status: TranscriptionStatus,
  target: DownloadTarget,
): TranscriptionStatus {
  return withTargetDownload(status, target, (total) => ({
    kind: 'downloading',
    received: 0,
    total,
  }));
}

/** `status` without the download of `target` (cancelled), until the core's event confirms it. */
export function withDownloadCleared(
  status: TranscriptionStatus,
  target: DownloadTarget,
): TranscriptionStatus {
  return withTargetDownload(status, target, () => null);
}

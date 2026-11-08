import type { DownloadTarget, TranscriptionPreferences, TranscriptionStatus } from '@se/protocol';
import { toast } from '@se/ui';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient, useCoreEvent } from '../../../shared/core';
import { useDesktopBridge } from '../../../shared/desktop';
import {
  cancelTranscriptionDownload,
  getTranscriptionStatus,
  importTranscriptionModel,
  removeTranscriptionFile,
  startTranscriptionDownload,
  updateTranscriptionPreferences,
} from '../api';
import { withDownloadCleared, withDownloadStarted } from '../services/transcription-downloads';
import { showError } from './show-error';

/** What « Réglages → Transcription » can do. Failures are shown as toasts. */
export interface TranscriptionActions {
  /** Saves a preference change (shown at once; restored if the core refuses it). */
  readonly save: (changes: Partial<TranscriptionPreferences>) => void;
  /** Starts a download the user consented to; progress arrives as core events. */
  readonly startDownload: (target: DownloadTarget) => void;
  readonly cancelDownload: (target: DownloadTarget) => void;
  /** Deletes an installed model, the voice model or the GPU pack. */
  readonly removeFile: (target: DownloadTarget) => void;
  /** Asks for a ggml `.bin` file and copies it into the models folder. */
  readonly importModel: () => void;
  /** Whether an imported file is being copied. */
  readonly importing: boolean;
  /** Reveals the models folder in the OS file manager. */
  readonly revealModelsDir: () => void;
}

/** The transcription status while its section is shown, and what can be done with it. */
export interface TranscriptionSettingsState extends TranscriptionActions {
  /** `null` while loading. */
  readonly status: TranscriptionStatus | null;
}

/**
 * Loads the transcription status when `active` becomes true (the section is shown) and follows
 * `transcriptionStatusChanged` events (download progress, imports, removals) while it stays so.
 * Preference saves are optimistic: the change shows until the core answers, even when a progress
 * event carrying the older preferences arrives in between.
 */
export function useTranscriptionSettings(active: boolean): TranscriptionSettingsState {
  const client = useCoreClient();
  const bridge = useDesktopBridge();
  const { t } = useTranslation();
  const [loaded, setLoaded] = useState<TranscriptionStatus | null>(null);
  const [pending, setPending] = useState<TranscriptionPreferences | null>(null);
  const [importing, setImporting] = useState(false);
  const fail = (error: unknown) => {
    showError(t, error);
  };
  useEffect(() => {
    if (!active) return;
    let current = true;
    getTranscriptionStatus(client)
      .then((status) => {
        if (current) setLoaded(status);
      })
      .catch((error: unknown) => {
        if (current) showError(t, error);
      });
    return () => {
      current = false;
    };
  }, [active, client, t]);
  useCoreEvent('transcriptionStatusChanged', (event) => {
    if (active) setLoaded(event.status);
  });
  const status = loaded === null || pending === null ? loaded : { ...loaded, preferences: pending };
  const reload = () => {
    getTranscriptionStatus(client).then(setLoaded).catch(fail);
  };
  const runImport = async () => {
    const path = await bridge.chooseFile({
      title: t('settings.transcription.models.importTitle'),
      buttonLabel: t('settings.transcription.models.importButton'),
    });
    if (path === null) return;
    setImporting(true);
    try {
      const imported = await importTranscriptionModel(client, path);
      setLoaded(imported.status);
      toast.success(t('settings.transcription.models.importDone', { name: imported.modelId }));
    } finally {
      setImporting(false);
    }
  };
  return {
    status,
    importing,
    save: (changes) => {
      if (status === null) return;
      const next = { ...status.preferences, ...changes };
      setPending(next);
      updateTranscriptionPreferences(client, next)
        .then(setLoaded)
        .catch(fail)
        .finally(() => {
          setPending((current) => (current === next ? null : current));
        });
    },
    startDownload: (target) => {
      setLoaded((current) => (current === null ? null : withDownloadStarted(current, target)));
      startTranscriptionDownload(client, target).catch((error: unknown) => {
        fail(error);
        reload();
      });
    },
    cancelDownload: (target) => {
      setLoaded((current) => (current === null ? null : withDownloadCleared(current, target)));
      cancelTranscriptionDownload(client, target).catch((error: unknown) => {
        fail(error);
        reload();
      });
    },
    removeFile: (target) => {
      removeTranscriptionFile(client, target).then(setLoaded).catch(fail);
    },
    importModel: () => {
      runImport().catch(fail);
    },
    revealModelsDir: () => {
      if (status !== null) bridge.showInFolder(status.modelsDir).catch(fail);
    },
  };
}

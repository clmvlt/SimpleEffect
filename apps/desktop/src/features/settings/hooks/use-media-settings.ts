import type { MediaSettings, MediaToolsStatus } from '@se/protocol';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { getMediaSettings, getMediaToolsStatus, updateMediaSettings } from '../api';
import { showError } from './show-error';

/** The media settings and tools status while the dialog is open, and how to change them. */
export interface MediaSettingsState {
  /** `null` while loading. */
  readonly settings: MediaSettings | null;
  /** `null` while checking (finding ffmpeg, detecting hardware encoders). */
  readonly tools: MediaToolsStatus | null;
  /** Saves a change (shown at once; restored if the core refuses it). */
  readonly save: (changes: Partial<MediaSettings>) => void;
}

/**
 * Loads the media settings and the tools status when `open` becomes true; `save` applies a change
 * optimistically, sends it, and checks the tools again when their folder changed.
 */
export function useMediaSettings(open: boolean): MediaSettingsState {
  const client = useCoreClient();
  const { t } = useTranslation();
  const [settings, setSettings] = useState<MediaSettings | null>(null);
  const [tools, setTools] = useState<MediaToolsStatus | null>(null);
  const fail = (error: unknown) => {
    showError(t, error);
  };
  const refreshTools = () => {
    setTools(null);
    getMediaToolsStatus(client).then(setTools).catch(fail);
  };
  useEffect(() => {
    if (!open) {
      return;
    }
    let current = true;
    const report = (error: unknown) => {
      if (current) showError(t, error);
    };
    getMediaSettings(client)
      .then((loaded) => {
        if (current) setSettings(loaded);
      })
      .catch(report);
    getMediaToolsStatus(client)
      .then((status) => {
        if (current) setTools(status);
      })
      .catch(report);
    return () => {
      current = false;
    };
  }, [open, client, t]);
  const save = (changes: Partial<MediaSettings>) => {
    if (settings === null) return;
    const previous = settings;
    const next = { ...settings, ...changes };
    setSettings(next);
    updateMediaSettings(client, next)
      .then((saved) => {
        setSettings(saved);
        if (saved.toolsDir !== previous.toolsDir) refreshTools();
      })
      .catch((error: unknown) => {
        setSettings(previous);
        fail(error);
      });
  };
  return { settings, tools, save };
}

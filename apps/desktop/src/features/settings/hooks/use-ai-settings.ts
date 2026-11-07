import type { AiSettings } from '@se/protocol';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { getAiSettings, updateAiSettings } from '../api';
import { showError } from './show-error';

/** The AI settings while the dialog is open, and how to change them. */
export interface AiSettingsState {
  /** `null` while loading. */
  readonly settings: AiSettings | null;
  /** Saves a change (shown at once; restored if the core refuses it). */
  readonly save: (changes: Partial<AiSettings>) => void;
}

/** Loads the AI settings when `open` becomes true; `save` applies a change optimistically. */
export function useAiSettings(open: boolean): AiSettingsState {
  const client = useCoreClient();
  const { t } = useTranslation();
  const [settings, setSettings] = useState<AiSettings | null>(null);
  useEffect(() => {
    if (!open) return;
    let current = true;
    getAiSettings(client)
      .then((loaded) => {
        if (current) setSettings(loaded);
      })
      .catch((error: unknown) => {
        if (current) showError(t, error);
      });
    return () => {
      current = false;
    };
  }, [open, client, t]);
  const save = (changes: Partial<AiSettings>) => {
    if (settings === null) return;
    const previous = settings;
    setSettings({ ...settings, ...changes });
    updateAiSettings(client, { ...settings, ...changes })
      .then(setSettings)
      .catch((error: unknown) => {
        setSettings(previous);
        showError(t, error);
      });
  };
  return { settings, save };
}

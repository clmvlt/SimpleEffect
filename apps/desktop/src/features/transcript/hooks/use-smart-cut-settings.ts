import type { SmartCutSettings } from '@se/protocol';
import { toast } from '@se/ui';
import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useStore } from 'zustand';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { getSmartCutSettings, updateSmartCutSettings } from '../api';
import { setSmartCutSettings, smartCutSettingsStore } from '../store/smart-cut-settings-store';

/** The smart-cut settings and how to save them. */
export interface SmartCutSettingsAccess {
  /** `null` while they are read. */
  readonly settings: SmartCutSettings | null;
  /** Saves `settings`; resolves with whether the core accepted them. */
  readonly save: (settings: SmartCutSettings) => Promise<boolean>;
}

/** The smart-cut settings, read from the core the first time a component needs them. */
export function useSmartCutSettings(): SmartCutSettingsAccess {
  const { t } = useTranslation();
  const client = useCoreClient();
  const settings = useStore(smartCutSettingsStore, (state) => state.settings);
  useEffect(() => {
    if (smartCutSettingsStore.getState().settings !== null) return;
    getSmartCutSettings(client).then(setSmartCutSettings, (error: unknown) => {
      toast.error(t(errorMessageKey(error)));
    });
  }, [client, t]);
  return {
    settings,
    save: async (next) => {
      try {
        setSmartCutSettings(await updateSmartCutSettings(client, next));
        return true;
      } catch (error: unknown) {
        toast.error(t(errorMessageKey(error)));
        return false;
      }
    },
  };
}

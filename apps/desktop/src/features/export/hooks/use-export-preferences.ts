import type { ExportPreferences } from '@se/protocol';
import { toast } from '@se/ui';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { exportsApi } from '../api';

/** The export preferences and how to change them. */
export interface ExportPreferencesState {
  /** `null` until read from the core. */
  readonly preferences: ExportPreferences | null;
  /** Saves new preferences (shown at once; a refusal is shown as a toast and undone). */
  readonly save: (next: ExportPreferences) => Promise<void>;
}

/** What the export dialog remembers (folder, file name template, saved presets), read once. */
export function useExportPreferences(): ExportPreferencesState {
  const client = useCoreClient();
  const { t } = useTranslation();
  const [preferences, setPreferences] = useState<ExportPreferences | null>(null);
  useEffect(() => {
    let current = true;
    exportsApi(client)
      .preferences()
      .then(
        (read) => {
          if (current) setPreferences(read);
        },
        (error: unknown) => {
          toast.error(t(errorMessageKey(error)));
        },
      );
    return () => {
      current = false;
    };
  }, [client, t]);
  const save = async (next: ExportPreferences) => {
    const previous = preferences;
    setPreferences(next);
    try {
      await exportsApi(client).savePreferences(next);
    } catch (error) {
      setPreferences(previous);
      toast.error(t(errorMessageKey(error)));
    }
  };
  return { preferences, save };
}

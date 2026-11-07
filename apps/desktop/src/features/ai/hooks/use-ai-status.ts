import type { AiStatus } from '@se/protocol';
import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useStore } from 'zustand';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { toast } from '@se/ui';
import { refreshAiStatus } from '../api';
import { aiStatusStore, setAiRefreshing, setAiStatus } from '../store/ai-status-store';
import { watchAiStatus } from '../services/watch-ai-status';

/** Keeps the app-wide AI status in step with the core. Mount once (the app layout does). */
export function useAiStatusSync(): void {
  const client = useCoreClient();
  useEffect(() => watchAiStatus(client, setAiStatus), [client]);
}

/** What is known about the AI provider (`null` until the core answered). */
export function useAiStatus(): AiStatus | null {
  return useStore(aiStatusStore, (state) => state.status);
}

/** Whether a refresh asked by the user is running. */
export function useAiRefreshing(): boolean {
  return useStore(aiStatusStore, (state) => state.refreshing);
}

/** Asks the core to check the AI provider again (errors are shown as a toast). */
export function useRefreshAiStatus(): () => void {
  const client = useCoreClient();
  const { t } = useTranslation();
  return () => {
    setAiRefreshing(true);
    refreshAiStatus(client)
      .then(setAiStatus)
      .catch((error: unknown) => {
        console.warn('[ai] refreshing the AI status failed', error);
        toast.error(t(errorMessageKey(error)));
      })
      .finally(() => {
        setAiRefreshing(false);
      });
  };
}

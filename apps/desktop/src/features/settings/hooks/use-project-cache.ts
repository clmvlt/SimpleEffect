import type { ProjectCacheInfo } from '@se/protocol';
import { toast } from '@se/ui';
import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { useDesktopBridge } from '../../../shared/desktop';
import { clearProjectCache, getProjectCache } from '../api';
import { showError } from './show-error';

/** The open project's cache and what can be done with it. */
export interface ProjectCacheState {
  /** `null` while loading (or without a project). */
  readonly cache: ProjectCacheInfo | null;
  readonly clearing: boolean;
  readonly reveal: () => void;
  readonly clear: () => Promise<void>;
}

/** Loads the cache of `projectId` when `open` becomes true. */
export function useProjectCache(projectId: string | null, open: boolean): ProjectCacheState {
  const client = useCoreClient();
  const bridge = useDesktopBridge();
  const { t } = useTranslation();
  const [cache, setCache] = useState<ProjectCacheInfo | null>(null);
  const [clearing, setClearing] = useState(false);
  const fail = (error: unknown) => {
    showError(t, error);
  };
  useEffect(() => {
    if (!open || projectId === null) {
      return;
    }
    let current = true;
    getProjectCache(client, projectId)
      .then((info) => {
        if (current) setCache(info);
      })
      .catch((error: unknown) => {
        if (current) showError(t, error);
      });
    return () => {
      current = false;
    };
  }, [open, projectId, client, t]);
  return {
    cache: projectId === null ? null : cache,
    clearing,
    reveal: () => {
      if (cache !== null) void bridge.showInFolder(cache.path).catch(fail);
    },
    clear: async () => {
      if (projectId === null) return;
      setClearing(true);
      try {
        setCache(await clearProjectCache(client, projectId));
        toast.success(t('settings.cache.cleared'));
      } catch (error) {
        fail(error);
      } finally {
        setClearing(false);
      }
    },
  };
}

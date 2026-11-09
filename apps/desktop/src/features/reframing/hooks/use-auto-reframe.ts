import type { ItemId, TrackingSettings } from '@se/protocol';
import { toast } from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient, useCoreEvent } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { selectProjectId, useProjectSelector } from '../../../shared/project-store';
import { reframingApi } from '../api';
import { reportNotice } from '../services/tracking';

/** Automatic reframing of clips, with its toasts. */
export interface AutoReframe {
  /** A request is being sent. */
  readonly busy: boolean;
  /** Makes `itemIds` follow their subject with `settings` (one undo step once applied). */
  reframe(itemIds: readonly ItemId[], settings: TrackingSettings): Promise<void>;
}

/** Automatic reframing in the open project (`null` while none is open). */
export function useAutoReframe(): AutoReframe | null {
  const { t } = useTranslation();
  const client = useCoreClient();
  const projectId = useProjectSelector(selectProjectId);
  const [busy, setBusy] = useState(false);
  if (projectId === null) return null;
  return {
    busy,
    reframe: async (itemIds, settings) => {
      setBusy(true);
      try {
        const notice = reportNotice(
          await reframingApi(client, projectId).reframe(itemIds, settings),
        );
        if (notice.kind === 'applied') {
          toast.success(t('editor.reframing.applied', { count: notice.count }));
        } else if (notice.kind === 'analysing') {
          toast.info(t('editor.reframing.analysing'), {
            description: t('editor.reframing.analysingHint'),
          });
        } else {
          toast.info(t('editor.reframing.nothing'));
        }
      } catch (error: unknown) {
        toast.error(t(errorMessageKey(error)));
      } finally {
        setBusy(false);
      }
    },
  };
}

/**
 * Tells the user when reframing that waited for face analyses finishes (mounted once by the
 * editor): the clips reframed, or the media that could not be analysed.
 */
export function useAutoReframeEvents(): void {
  const { t } = useTranslation();
  const projectId = useProjectSelector(selectProjectId);
  useCoreEvent('autoReframeFinished', (event) => {
    if (event.projectId !== projectId) return;
    if (event.appliedItemIds.length > 0) {
      toast.success(t('editor.reframing.finished', { count: event.appliedItemIds.length }));
    }
    const [failure] = event.failures;
    if (failure !== undefined) {
      toast.error(t('editor.reframing.failed', { count: event.failures.length }), {
        description: failure.message,
      });
    }
  });
}

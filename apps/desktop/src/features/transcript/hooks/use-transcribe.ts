import { CoreRequestError, type AssetId, type ProjectId } from '@se/protocol';
import { toast } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { openSettings } from '../../settings';
import { transcribeAssets } from '../api';

/**
 * Queues the transcription of assets. Without a model (or without whisper.cpp) the toast offers
 * « Réglages » (the transcription section), where models are downloaded.
 */
export function useTranscribe(
  projectId: ProjectId | null,
): (assetIds: readonly AssetId[]) => Promise<void> {
  const client = useCoreClient();
  const { t } = useTranslation();
  return async (assetIds) => {
    if (projectId === null || assetIds.length === 0) return;
    try {
      await transcribeAssets(client, projectId, assetIds);
    } catch (error) {
      const needsSettings =
        error instanceof CoreRequestError &&
        (error.code === 'NO_TRANSCRIPTION_MODEL' || error.code === 'TRANSCRIPTION_UNAVAILABLE');
      toast.error(t(errorMessageKey(error)), {
        action: needsSettings
          ? {
              label: t('editor.transcript.openSettings'),
              onClick: () => {
                openSettings('transcription');
              },
            }
          : undefined,
      });
    }
  };
}

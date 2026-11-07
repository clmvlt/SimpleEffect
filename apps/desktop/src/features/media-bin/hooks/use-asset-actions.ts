import type { Asset, DerivedKind } from '@se/protocol';
import { toast } from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { useDesktopBridge } from '../../../shared/desktop';
import { errorMessageKey } from '../../../shared/errors';
import { generateDerivedMedia, relinkAsset, removeAsset } from '../api';
import { assetName } from '../services/asset-list';
import { MEDIA_EXTENSIONS } from '../services/media-extensions';

/** A relink waiting for the user's confirmation (the file's content differs). */
export interface PendingRelink {
  readonly asset: Asset;
  readonly path: string;
}

/**
 * What can be done with an asset: relink (with confirmation for other content), remove, reveal,
 * generate a proxy or retry a failed mezzanine or proxy.
 */
export function useAssetActions(projectId: string | null) {
  const client = useCoreClient();
  const bridge = useDesktopBridge();
  const { t } = useTranslation();
  const [pendingRelink, setPendingRelink] = useState<PendingRelink | null>(null);

  const fail = (error: unknown) => {
    console.warn('[media-bin]', error);
    toast.error(t(errorMessageKey(error)));
  };

  const relinkTo = async (asset: Asset, path: string, force: boolean): Promise<void> => {
    if (projectId === null) return;
    try {
      const outcome = await relinkAsset(client, { projectId, assetId: asset.id, path, force });
      if (outcome === 'contentDiffers') {
        setPendingRelink({ asset, path });
      } else {
        toast.success(t('editor.media.toasts.relinked', { name: assetName(asset) }));
      }
    } catch (error) {
      fail(error);
    }
  };

  return {
    pendingRelink,
    relink: async (asset: Asset): Promise<void> => {
      const [path] = await bridge.chooseFiles({
        title: t('editor.media.relinkDialog.title', { name: assetName(asset) }),
        buttonLabel: t('editor.media.relinkDialog.button'),
        filterName: t('editor.media.dialog.filter'),
        allFilesName: t('editor.media.dialog.allFiles'),
        extensions: MEDIA_EXTENSIONS,
      });
      if (path !== undefined) await relinkTo(asset, path, false);
    },
    confirmRelink: async (): Promise<void> => {
      const pending = pendingRelink;
      setPendingRelink(null);
      if (pending !== null) await relinkTo(pending.asset, pending.path, true);
    },
    cancelRelink: (): void => {
      setPendingRelink(null);
    },
    remove: async (asset: Asset): Promise<void> => {
      if (projectId === null) return;
      try {
        await removeAsset(client, projectId, asset.id);
        toast.success(t('editor.media.toasts.removed', { name: assetName(asset) }));
      } catch (error) {
        fail(error);
      }
    },
    reveal: (asset: Asset): void => {
      void bridge.showInFolder(asset.originalPath).catch(fail);
    },
    generateDerived: async (asset: Asset, derived: DerivedKind): Promise<void> => {
      if (projectId === null) return;
      try {
        await generateDerivedMedia(client, { projectId, assetId: asset.id, derived });
        toast.success(t(`editor.media.toasts.${derived}Queued`, { name: assetName(asset) }));
      } catch (error) {
        fail(error);
      }
    },
  };
}

import type { Asset } from '@se/protocol';
import {
  Button,
  EmptyState,
  IconButton,
  Panel,
  PanelActions,
  PanelBody,
  PanelHeader,
  PanelTitle,
  Tooltip,
} from '@se/ui';
import { Film, Plus, SearchX } from '@se/ui/icons';
import { useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useStore } from 'zustand';

import { selectProjectId, useProjectSelector } from '../../../shared/project-store';
import { useAssetActions } from '../hooks/use-asset-actions';
import { useDecodabilityReporter } from '../hooks/use-decodability-reporter';
import { useJobsSync } from '../hooks/use-jobs-sync';
import { useMediaAssets } from '../hooks/use-media-assets';
import { useMediaImport } from '../hooks/use-media-import';
import { mediaBinStore, updateMediaBin } from '../store/media-bin-store';
import type { AssetMenuHandlers } from './AssetMenuItems';
import { MediaBinToolbar } from './MediaBinToolbar';
import { MediaCard } from './MediaCard';
import { MediaDropZone } from './MediaDropZone';
import { MediaInfoPanel } from './MediaInfoPanel';
import { MediaRow } from './MediaRow';
import { RelinkConfirmDialog } from './RelinkConfirmDialog';

/**
 * The project's media bin: import (button or files dropped from the OS), grid or list of the
 * assets with previews, filter by kind, search by name, job progress, missing media with
 * « Relier… », and the details of the selected asset. It also keeps the jobs store current and
 * reports what WebCodecs can decode.
 */
export function MediaBinPanel() {
  const { t } = useTranslation();
  const projectId = useProjectSelector(selectProjectId);
  const { all, visible } = useMediaAssets();
  const { view, selectedAssetId } = useStore(mediaBinStore);
  const importer = useMediaImport(projectId);
  const actions = useAssetActions(projectId);
  useJobsSync(projectId);
  useDecodabilityReporter(projectId, all);
  useEffect(() => {
    // Details belong to the project they were opened in.
    updateMediaBin({ selectedAssetId: null });
  }, [projectId]);
  const selected = all.find((asset) => asset.id === selectedAssetId) ?? null;
  const handlers: AssetMenuHandlers = {
    onDetails: (asset: Asset) => {
      updateMediaBin({ selectedAssetId: asset.id });
    },
    onRelink: (asset) => void actions.relink(asset),
    onReveal: actions.reveal,
    onGenerateProxy: (asset) => void actions.generateDerived(asset, 'proxy'),
    onRemove: (asset) => void actions.remove(asset),
  };
  const importLabel = t('editor.media.toolbar.import');
  const importFromDialog = () => void importer.importWithDialog();
  const Item = view === 'grid' ? MediaCard : MediaRow;
  return (
    <Panel>
      <PanelHeader>
        <PanelTitle>{t('editor.media.title')}</PanelTitle>
        <PanelActions>
          <Tooltip content={importLabel}>
            <IconButton
              icon={Plus}
              size="xs"
              label={importLabel}
              loading={importer.importing}
              disabled={projectId === null}
              onClick={importFromDialog}
            />
          </Tooltip>
        </PanelActions>
      </PanelHeader>
      <MediaDropZone onDropFiles={(files) => void importer.importDropped(files)}>
        {all.length > 0 && <MediaBinToolbar />}
        <PanelBody scrollable={all.length > 0}>
          {all.length === 0 ? (
            <EmptyState
              icon={Film}
              title={t('editor.media.empty.title')}
              description={t('editor.media.empty.description')}
              actions={
                <Button size="sm" disabled={projectId === null} onClick={importFromDialog}>
                  {t('editor.media.empty.action')}
                </Button>
              }
            />
          ) : visible.length === 0 ? (
            <EmptyState
              icon={SearchX}
              title={t('editor.media.noMatch.title')}
              description={t('editor.media.noMatch.description')}
            />
          ) : (
            <ul
              className={
                view === 'grid'
                  ? 'grid grid-cols-[repeat(auto-fill,minmax(152px,1fr))] gap-1 p-1'
                  : 'flex flex-col gap-0.5 p-1'
              }
            >
              {visible.map((asset) => (
                <Item
                  key={asset.id}
                  asset={asset}
                  selected={asset.id === selectedAssetId}
                  {...handlers}
                />
              ))}
            </ul>
          )}
        </PanelBody>
        {selected !== null && (
          <MediaInfoPanel
            asset={selected}
            onClose={() => {
              updateMediaBin({ selectedAssetId: null });
            }}
            onRelink={handlers.onRelink}
            onRetryDerived={(asset, derived) => void actions.generateDerived(asset, derived)}
          />
        )}
      </MediaDropZone>
      <RelinkConfirmDialog
        pending={actions.pendingRelink}
        onConfirm={() => void actions.confirmRelink()}
        onCancel={actions.cancelRelink}
      />
    </Panel>
  );
}

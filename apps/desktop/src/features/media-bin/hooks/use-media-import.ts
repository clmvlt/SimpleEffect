import { toast } from '@se/ui';
import type { TFunction } from 'i18next';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { useDesktopBridge } from '../../../shared/desktop';
import { errorMessageKey } from '../../../shared/errors';
import { importMedia } from '../api';
import { droppedPaths, summarizeImport, type ImportSummary } from '../services/import-summary';
import { MEDIA_EXTENSIONS } from '../services/media-extensions';

/** Toasts telling what an import did. */
function notify(t: TFunction, summary: ImportSummary): void {
  if (summary.imported > 0) {
    toast.success(t('editor.media.toasts.imported', { count: summary.imported }));
  }
  if (summary.duplicates > 0) {
    toast.info(t('editor.media.toasts.duplicates', { count: summary.duplicates }));
  }
  for (const failure of summary.failures) {
    toast.error(t('editor.media.toasts.failed', { name: failure.name }), {
      description: t(`errors.${failure.code}`),
    });
  }
}

/** Importing media into the open project: from the native dialog or files dropped from the OS. */
export function useMediaImport(projectId: string | null) {
  const client = useCoreClient();
  const bridge = useDesktopBridge();
  const { t } = useTranslation();
  const [importing, setImporting] = useState(false);

  const importPaths = async (paths: readonly string[]): Promise<void> => {
    if (projectId === null || paths.length === 0) {
      return;
    }
    setImporting(true);
    try {
      notify(t, summarizeImport(await importMedia(client, projectId, paths)));
    } catch (error) {
      toast.error(t(errorMessageKey(error)));
    } finally {
      setImporting(false);
    }
  };

  return {
    importing,
    importWithDialog: async (): Promise<void> => {
      const paths = await bridge.chooseFiles({
        title: t('editor.media.dialog.title'),
        buttonLabel: t('editor.media.dialog.button'),
        filterName: t('editor.media.dialog.filter'),
        allFilesName: t('editor.media.dialog.allFiles'),
        extensions: MEDIA_EXTENSIONS,
      });
      await importPaths(paths);
    },
    importDropped: (files: FileList): Promise<void> =>
      importPaths(droppedPaths(files, (file) => bridge.getPathForFile(file))),
  };
}

import type { LibraryImportResult, LibraryTemplate, MotionTemplate } from '@se/protocol';
import { toast } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { useDesktopBridge } from '../../../shared/desktop';
import { errorMessageKey } from '../../../shared/errors';
import { useProjectMirrorReader, useProjectSelector } from '../../../shared/project-store';
import { showSidePanelTab } from '../../../shared/side-panel';
import {
  playheadStore,
  playheadTimeOf,
  selectItems,
  selectionStore,
  useActiveSequence,
} from '../../timeline';
import { libraryApi, templatesApi } from '../api';
import { captionStylePatchOf } from '../services/browser-entries';
import { openTemplateCode } from '../store/code-dialog-store';
import { openRenameEntry, openSaveTemplate } from '../store/library-dialog-store';

/** File extension of exported templates. */
export const TEMPLATE_FILE_EXTENSION = 'setemplate';

/** What the template browser's cards and toolbar do (each failure shown as a toast). */
export interface BrowserActions {
  /** Whether a project and sequence are open (insertions and saves need them). */
  readonly ready: boolean;
  readonly insertProject: (template: MotionTemplate) => void;
  readonly insertLibrary: (entry: LibraryTemplate) => void;
  readonly saveTemplate: (template: MotionTemplate, linked: LibraryTemplate | null) => void;
  readonly update: (template: MotionTemplate, entry: LibraryTemplate) => void;
  readonly viewCode: (template: MotionTemplate) => void;
  readonly removeProject: (template: MotionTemplate) => void;
  readonly applyCaptionStyle: (entry: LibraryTemplate) => void;
  readonly exportEntry: (entry: LibraryTemplate) => void;
  readonly renameEntry: (entry: LibraryTemplate) => void;
  readonly removeEntry: (entry: LibraryTemplate) => void;
  readonly importFiles: () => void;
}

/** The browser's actions on the open project and the library. */
export function useBrowserActions(): BrowserActions {
  const { t } = useTranslation();
  const client = useCoreClient();
  const bridge = useDesktopBridge();
  const projectId = useProjectSelector((state) => state.mirror?.projectId ?? null);
  const readMirror = useProjectMirrorReader();
  const sequence = useActiveSequence();
  const api = projectId === null ? null : templatesApi(client, projectId);
  const library = libraryApi(client);
  const fail = (error: unknown) => {
    toast.error(t(errorMessageKey(error)));
  };
  const placed = (itemId: string) => {
    selectItems([itemId]);
    showSidePanelTab('inspector');
  };
  const playhead = () =>
    sequence === null ? null : playheadTimeOf(playheadStore.getState(), sequence.id);
  const announceImports = (results: readonly LibraryImportResult[]) => {
    for (const { path, outcome } of results) {
      if (outcome.kind === 'failed') {
        toast.error(t('editor.templates.browser.importFailed', { path, message: outcome.message }));
      } else {
        toast.success(
          t(`editor.templates.browser.import.${outcome.kind}`, {
            name: outcome.template.name,
            version: outcome.template.version,
          }),
        );
      }
    }
  };
  return {
    ready: api !== null && sequence !== null,
    insertProject: (template) => {
      const at = playhead();
      if (api === null || sequence === null || at === null) return;
      api
        .insert(sequence.id, template.id, at, template.defaultDuration)
        .then((result) => {
          placed(result.itemId);
        })
        .catch(fail);
    },
    insertLibrary: (entry) => {
      const at = playhead();
      if (api === null || sequence === null || at === null) return;
      api
        .insertFromLibrary(sequence.id, entry.id, at)
        .then((result) => {
          placed(result.itemId);
        })
        .catch(fail);
    },
    saveTemplate: openSaveTemplate,
    update: (template, entry) => {
      api
        ?.updateFromLibrary(template.id)
        .then((report) => {
          toast.success(
            t('editor.templates.browser.updated', {
              name: entry.name,
              version: report.libraryVersion,
              count: report.reset.length,
            }),
          );
        })
        .catch(fail);
    },
    viewCode: (template) => {
      openTemplateCode(template.id);
    },
    removeProject: (template) => {
      api?.remove(template.id).catch(fail);
    },
    applyCaptionStyle: (entry) => {
      const mirror = readMirror();
      if (entry.content.kind !== 'captionStyle' || mirror === null || projectId === null) return;
      const captions = selectionStore
        .getState()
        .itemIds.filter((id) => mirror.items[id]?.kind.kind === 'caption');
      if (captions.length === 0) {
        toast.info(t('editor.templates.browser.selectCaptions'));
        return;
      }
      client
        .send({
          kind: 'updateCaptionStyle',
          projectId,
          itemIds: [...captions],
          patch: captionStylePatchOf(entry.content.style),
        })
        .then(() => {
          toast.success(
            t('editor.templates.browser.styleApplied', {
              name: entry.name,
              count: captions.length,
            }),
          );
        })
        .catch(fail);
    },
    exportEntry: (entry) => {
      void bridge
        .chooseSaveFile({
          title: t('editor.templates.browser.exportTitle'),
          buttonLabel: t('editor.templates.browser.exportButton'),
          defaultName: `${entry.name}.${TEMPLATE_FILE_EXTENSION}`,
          filterName: t('editor.templates.browser.fileFilter'),
          extensions: [TEMPLATE_FILE_EXTENSION],
        })
        .then(async (path) => {
          if (path === null) return;
          await library.exportFile(entry.id, path);
          toast.success(t('editor.templates.browser.exported', { name: entry.name }));
        })
        .catch(fail);
    },
    renameEntry: openRenameEntry,
    removeEntry: (entry) => {
      library
        .remove(entry.id)
        .then(() => {
          toast.success(t('editor.templates.browser.removedFromLibrary', { name: entry.name }));
        })
        .catch(fail);
    },
    importFiles: () => {
      if (api === null) return;
      void bridge
        .chooseFiles({
          title: t('editor.templates.browser.importTitle'),
          buttonLabel: t('editor.templates.browser.importButton'),
          filterName: t('editor.templates.browser.fileFilter'),
          extensions: [TEMPLATE_FILE_EXTENSION],
          allFilesName: t('editor.templates.browser.allFiles'),
        })
        .then(async (paths) => {
          if (paths.length === 0) return;
          announceImports(await api.importFiles(paths));
        })
        .catch(fail);
    },
  };
}

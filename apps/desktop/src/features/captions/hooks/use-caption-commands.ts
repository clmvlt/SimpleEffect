import type {
  CaptionStyleChoice,
  CaptionStylePatch,
  CaptionsReport,
  Flicks,
  ItemId,
  SequenceEntity,
  TimeRange,
} from '@se/protocol';
import { toast } from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { useDesktopBridge } from '../../../shared/desktop';
import { errorMessageKey } from '../../../shared/errors';
import { selectProjectId, useProjectSelector } from '../../../shared/project-store';
import { selectItems } from '../../timeline';
import { captionsApi, type CaptionsApi } from '../api';

/** The caption actions of the panel, each one command (one undo step) with its toasts. */
export interface CaptionCommands {
  readonly busy: boolean;
  generate(range: TimeRange | null, style: CaptionStyleChoice | null): Promise<void>;
  importFile(style: CaptionStyleChoice | null): Promise<void>;
  exportFile(): Promise<void>;
  editText(itemId: ItemId, text: string): Promise<boolean>;
  setEmphasis(itemId: ItemId, words: readonly number[], emphasis: boolean): Promise<void>;
  merge(itemIds: readonly ItemId[]): Promise<void>;
  split(itemId: ItemId, wordIndex: number): Promise<void>;
  shift(itemIds: readonly ItemId[], delta: Flicks): Promise<void>;
  updateStyle(itemIds: readonly ItemId[], patch: CaptionStylePatch): Promise<void>;
  applyPreset(itemIds: readonly ItemId[], preset: string): Promise<void>;
}

/** The caption commands of the open project on `sequence` (`null` while none is shown). */
export function useCaptionCommands(sequence: SequenceEntity | null): CaptionCommands | null {
  const { t } = useTranslation();
  const client = useCoreClient();
  const bridge = useDesktopBridge();
  const projectId = useProjectSelector(selectProjectId);
  const [busy, setBusy] = useState(false);
  if (projectId === null || sequence === null) return null;
  const api: CaptionsApi = captionsApi(client, projectId);
  const trackName = t('editor.captions.trackName');
  const attempt = async <T>(work: () => Promise<T>): Promise<T | null> => {
    setBusy(true);
    try {
      return await work();
    } catch (error: unknown) {
      toast.error(t(errorMessageKey(error)));
      return null;
    } finally {
      setBusy(false);
    }
  };
  const announce = (report: CaptionsReport | null) => {
    if (report === null) return;
    if (report.itemIds.length === 0) {
      toast.info(
        t(
          report.untranscribedAssets.length > 0
            ? 'editor.captions.untranscribed'
            : 'editor.captions.nothingSpoken',
        ),
      );
      return;
    }
    toast.success(t('editor.captions.made', { count: report.itemIds.length }));
  };
  return {
    busy,
    generate: async (range, style) => {
      announce(await attempt(() => api.generate(sequence.id, range, style, trackName)));
    },
    importFile: async (style) => {
      const [path] = await bridge.chooseFiles({
        title: t('editor.captions.import.title'),
        buttonLabel: t('editor.captions.import.button'),
        filterName: t('editor.captions.srtFilter'),
        extensions: ['srt'],
        allFilesName: t('editor.captions.allFiles'),
      });
      if (path === undefined) return;
      announce(await attempt(() => api.importSubtitles(sequence.id, path, style, trackName)));
    },
    exportFile: async () => {
      const path = await bridge.chooseSaveFile({
        title: t('editor.captions.export.title'),
        buttonLabel: t('editor.captions.export.button'),
        defaultName: `${sequence.name}.srt`,
        filterName: t('editor.captions.srtFilter'),
        extensions: ['srt'],
      });
      if (path === null) return;
      const count = await attempt(() => api.exportSubtitles(sequence.id, path));
      if (count !== null) toast.success(t('editor.captions.exported', { count }));
    },
    editText: async (itemId, text) => (await attempt(() => api.editText(itemId, text))) !== null,
    setEmphasis: async (itemId, words, emphasis) => {
      await attempt(() => api.setEmphasis(itemId, words, emphasis));
    },
    merge: async (itemIds) => {
      const merged = await attempt(() => api.merge(itemIds));
      if (merged !== null) selectItems([merged]);
    },
    split: async (itemId, wordIndex) => {
      const right = await attempt(() => api.split(itemId, wordIndex));
      if (right !== null) selectItems([right]);
    },
    shift: async (itemIds, delta) => {
      await attempt(() => api.shift(itemIds, delta));
    },
    updateStyle: async (itemIds, patch) => {
      await attempt(() => api.updateStyle(itemIds, patch));
    },
    applyPreset: async (itemIds, preset) => {
      await attempt(() => api.applyPreset(itemIds, preset));
    },
  };
}

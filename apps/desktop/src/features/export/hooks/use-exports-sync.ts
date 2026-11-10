import type { ExportInfo } from '@se/protocol';
import { toast } from '@se/ui';
import type { TFunction } from 'i18next';
import { useEffect, useEffectEvent } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { useDesktopBridge, type SimpleEffectBridge } from '../../../shared/desktop';
import { byteSize } from '../../../shared/text';
import {
  exportIdOfTag,
  exportNotice,
  exportNotificationTag,
  fileNameOf,
  type ExportNotice,
} from '../services/export-notice';
import { connectExportsSync } from '../services/exports-sync';
import { exportsStore } from '../store/exports-store';

/** Whether the app window has the focus (system notifications are for when it has not). */
function windowFocused(): boolean {
  return typeof document === 'undefined' || document.hasFocus();
}

/** Reveals an export's file in the OS file manager. */
function revealExport(bridge: SimpleEffectBridge, info: ExportInfo): void {
  bridge.showInFolder(info.path).catch((error: unknown) => {
    console.warn('[export] revealing an export failed', error);
  });
}

/** The title and text announcing an ended export. */
function noticeTexts(notice: ExportNotice, t: TFunction, language: string) {
  const name = fileNameOf(notice.export.path);
  if (notice.tone === 'error') {
    return { title: t('export.notice.failed', { name }), description: notice.message };
  }
  const size = byteSize(notice.sizeBytes, language);
  return {
    title: t('export.notice.completed', { name }),
    description: notice.warning ?? t('export.notice.size', { size }),
  };
}

/** A toast, plus a system notification when the window is not focused. */
function announce(
  notice: ExportNotice,
  bridge: SimpleEffectBridge,
  t: TFunction,
  language: string,
) {
  const { title, description } = noticeTexts(notice, t, language);
  if (notice.tone === 'error') {
    toast.error(title, { description });
  } else {
    const action = {
      label: t('export.notice.reveal'),
      onClick: () => {
        revealExport(bridge, notice.export);
      },
    };
    if (notice.warning === null) toast.success(title, { description, action });
    else toast.warning(title, { description, action });
  }
  if (windowFocused()) return;
  const tag = exportNotificationTag(notice.export);
  bridge.showNotification({ title, body: description, tag }).catch((error: unknown) => {
    console.warn('[export] showing a system notification failed', error);
  });
}

/**
 * Keeps the core's exports mirrored for the whole app and announces each end: a toast with
 * « Afficher dans le dossier », plus a system notification when the window is not focused, whose
 * click reveals the file.
 */
export function useExportsSync(): void {
  const client = useCoreClient();
  const bridge = useDesktopBridge();
  const { t, i18n } = useTranslation();
  const onChange = useEffectEvent((previous: ExportInfo | undefined, next: ExportInfo) => {
    const notice = exportNotice(previous, next);
    if (notice !== null) announce(notice, bridge, t, i18n.language);
  });
  const onNotificationClick = useEffectEvent((tag: string) => {
    const exportId = exportIdOfTag(tag);
    const info = exportsStore.getState().exports.find((known) => known.id === exportId);
    if (info?.state.kind === 'completed') revealExport(bridge, info);
  });
  useEffect(() => connectExportsSync(client, onChange), [client]);
  useEffect(() => bridge.onNotificationClick(onNotificationClick), [bridge]);
}

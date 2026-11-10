import { toast } from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { useDesktopBridge } from '../../../shared/desktop';
import { errorMessageKey } from '../../../shared/errors';
import { byteSize } from '../../../shared/text';
import { exportDiagnostics } from '../api';
import { browserProbes } from '../services/browser-probes';
import { collectClientDiagnostics, diagnosticsFileName } from '../services/client-diagnostics';
import { useRuntimeInfo } from './use-runtime-info';

/** « Exporter un diagnostic »: whether one is being written, and how to start one. */
export interface DiagnosticsExport {
  readonly exporting: boolean;
  readonly start: () => void;
}

/**
 * Asks where to save the archive, collects the renderer's part (versions, GPU, codecs), lets the
 * core write the zip, then shows a toast that reveals it.
 */
export function useExportDiagnostics(): DiagnosticsExport {
  const { t, i18n } = useTranslation();
  const client = useCoreClient();
  const bridge = useDesktopBridge();
  const runtime = useRuntimeInfo();
  const [exporting, setExporting] = useState(false);
  const run = async () => {
    const path = await bridge.chooseSaveFile({
      title: t('about.diagnostics.dialogTitle'),
      buttonLabel: t('about.diagnostics.dialogButton'),
      defaultName: diagnosticsFileName(new Date()),
      filterName: t('about.diagnostics.filterName'),
      extensions: ['zip'],
    });
    if (path === null) return;
    setExporting(true);
    const diagnostics = await collectClientDiagnostics(
      { app: __APP_VERSION__, engine: __ENGINE_VERSION__, runtime },
      browserProbes(),
    );
    const summary = await exportDiagnostics(client, path, diagnostics);
    toast.success(t('about.diagnostics.done', { size: byteSize(summary.bytes, i18n.language) }), {
      action: {
        label: t('about.diagnostics.reveal'),
        onClick: () => {
          void bridge.showInFolder(summary.path);
        },
      },
    });
  };
  const start = () => {
    run()
      .catch((error: unknown) => {
        console.warn('[about] diagnostics export failed', error);
        toast.error(t(errorMessageKey(error)));
      })
      .finally(() => {
        setExporting(false);
      });
  };
  return { exporting, start };
}

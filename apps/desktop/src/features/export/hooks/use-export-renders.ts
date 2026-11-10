import type { CoreClient, ExportId, ExportRenderRequest } from '@se/protocol';
import { toast } from '@se/ui';
import { useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient, useCoreEvent } from '../../../shared/core';
import { catalogFaces, fontCatalogStore, loadFontCatalog } from '../../../shared/fonts';
import { exportsApi } from '../api';
import { renderIssueCount } from '../services/export-notice';
import {
  startExportRender,
  type ExportRender,
  type ExportRenderDeps,
} from '../services/export-render';

/** The engine's export session, loaded only when an export runs. */
const startSession: ExportRenderDeps['startSession'] = async (options) => {
  const engine = await import('@se/engine/export');
  return engine.startExportSession(options);
};

function renderDeps(client: CoreClient, onIssues: ExportRenderDeps['onIssues']): ExportRenderDeps {
  const api = exportsApi(client);
  return {
    endpoint: () => client.endpoint,
    fonts: async (endpoint) => {
      await loadFontCatalog(client).catch(() => undefined);
      return catalogFaces(fontCatalogStore.getState().families ?? [], endpoint);
    },
    startSession,
    reportProgress: api.reportProgress,
    finish: api.finishRender,
    fail: api.failRender,
    onIssues,
    now: () => performance.now(),
  };
}

/**
 * Draws the exports the core asks for (`exportRenderRequested`, ADR-0038) for as long as the app
 * runs, whatever page is shown: the request carries its own snapshot. `exportRenderCancelled`
 * stops the matching render; a finished export that could not draw everything says so in a toast.
 */
export function useExportRenders(deps: Partial<ExportRenderDeps> = {}): void {
  const client = useCoreClient();
  const { t } = useTranslation();
  const renders = useRef(new Map<ExportId, ExportRender>());
  useCoreEvent('exportRenderRequested', (request: ExportRenderRequest) => {
    const onIssues: ExportRenderDeps['onIssues'] = (_exportId, issues) => {
      const count = renderIssueCount(issues);
      if (count > 0) toast.warning(t('export.notice.issues', { count }));
    };
    const render = startExportRender(request, { ...renderDeps(client, onIssues), ...deps });
    renders.current.set(request.exportId, render);
    void render.done.finally(() => {
      if (renders.current.get(request.exportId) === render)
        renders.current.delete(request.exportId);
    });
  });
  useCoreEvent('exportRenderCancelled', ({ exportId }) => {
    renders.current.get(exportId)?.cancel();
  });
  useEffect(() => {
    const running = renders.current;
    return () => {
      for (const render of running.values()) render.cancel();
      running.clear();
    };
  }, []);
}

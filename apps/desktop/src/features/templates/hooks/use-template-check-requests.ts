import type { TemplateCheckRequest } from '@se/protocol';
import { useRef } from 'react';

import { useCoreClient, useCoreEndpoint, useCoreEvent } from '../../../shared/core';
import { catalogFaces, fontCatalogStore, loadFontCatalog } from '../../../shared/fonts';
import { templatesSupported } from '../../../shared/templates';
import { answerTemplateCheck, type TemplateCheckDeps } from '../services/template-checks';

/** The real check: `@se/engine/templates` (the compiler and the sandbox) loads only when needed. */
const check: TemplateCheckDeps['check'] = async (request, families, endpoint) => {
  if (!templatesSupported()) throw new Error('this window cannot run Workers');
  const templates = await import('@se/engine/templates');
  return templates.checkTemplate(request.source, {
    width: request.width,
    height: request.height,
    pictureWidth: request.pictureWidth,
    pictureHeight: request.pictureHeight,
    ...(endpoint === null
      ? {}
      : { fonts: templates.catalogFontSource(catalogFaces(families, endpoint)) }),
  });
};

/**
 * Answers the motion template checks of AI agents (`templateCheckRequested`, prompt 24) for as
 * long as the app runs, one at a time: the module is compiled, run in a sandboxed Worker and
 * judged here, then the verdict goes back to the core.
 */
export function useTemplateCheckRequests(deps: Partial<TemplateCheckDeps> = {}): void {
  const client = useCoreClient();
  const endpoint = useCoreEndpoint();
  const queue = useRef<Promise<void>>(Promise.resolve());
  useCoreEvent('templateCheckRequested', (request: TemplateCheckRequest) => {
    const fonts = loadFontCatalog(client).catch(() => undefined);
    const resolved: TemplateCheckDeps = {
      endpoint: () => endpoint,
      families: () => fontCatalogStore.getState().families,
      check,
      report: async (requestId, outcome) => {
        await client.send({ kind: 'reportTemplateCheck', requestId, outcome });
      },
      ...deps,
    };
    queue.current = queue.current.then(async () => {
      await fonts;
      await answerTemplateCheck(request, resolved);
    });
  });
}

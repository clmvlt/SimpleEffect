import { createFontRegistry } from '@se/engine/fonts';
import type { FrameRenderRequest } from '@se/protocol';
import { useRef } from 'react';

import { useCoreClient, useCoreEndpoint, useCoreEvent } from '../../../shared/core';
import { catalogFaces, fontCatalogStore, loadFontCatalog } from '../../../shared/fonts';
import { useProjectMirrorReader } from '../../../shared/project-store';
import { templateRuntimeFor } from '../../../shared/templates';
import { answerFrameRequest, type AgentFrameDeps } from '../services/agent-frames';
import { PreviewUnsupportedError } from '../services/preview-engine-factory';

/** The real still renderer: `@se/engine/preview` (PixiJS + Mediabunny) loads only when needed. */
const createStills: AgentFrameDeps['createStills'] = async (options) => {
  if (typeof VideoDecoder === 'undefined' || typeof OffscreenCanvas === 'undefined') {
    throw new PreviewUnsupportedError();
  }
  const { createStillRenderer } = await import('@se/engine/preview');
  return createStillRenderer(options);
};

/** A font registry over the document's font set, or `null` where fonts cannot load (jsdom). */
const fonts: AgentFrameDeps['fonts'] = (families, endpoint) => {
  if (typeof FontFace === 'undefined' || typeof document === 'undefined') return null;
  const registry = createFontRegistry();
  registry.setCatalog(catalogFaces(families, endpoint));
  return registry;
};

/** The app's template runtime, or `null` where Workers cannot run (jsdom). */
const templates: AgentFrameDeps['templates'] = async (families, endpoint) =>
  templateRuntimeFor({ endpoint, families }).catch(() => null);

/** Sends a JPEG to the core's upload route. */
const upload: AgentFrameDeps['upload'] = async (url, jpeg) => {
  const response = await fetch(url, {
    method: 'PUT',
    headers: { 'Content-Type': 'image/jpeg' },
    body: jpeg,
  });
  if (!response.ok) throw new Error(`the core refused a frame (${String(response.status)})`);
};

/**
 * Answers the frame requests of AI agents (`renderFramesRequested`, ADR-0028) for as long as the
 * app runs: the frames of the project shown in the editor are drawn offscreen with the viewer's
 * engine and uploaded; other requests are refused at once so agents learn why. Requests are
 * answered one at a time.
 */
export function useAgentFrameRequests(deps: Partial<AgentFrameDeps> = {}): void {
  const client = useCoreClient();
  const endpoint = useCoreEndpoint();
  const readMirror = useProjectMirrorReader();
  const queue = useRef<Promise<void>>(Promise.resolve());
  useCoreEvent('renderFramesRequested', (request: FrameRenderRequest) => {
    void loadFontCatalog(client).catch(() => undefined);
    const resolved: AgentFrameDeps = {
      mirror: readMirror,
      endpoint: () => endpoint,
      families: () => fontCatalogStore.getState().families,
      fonts,
      templates,
      createStills,
      upload,
      reportFailure: async (requestId, message) => {
        await client.send({ kind: 'reportFrameRenderFailure', requestId, message });
      },
      ...deps,
    };
    queue.current = queue.current.then(() => answerFrameRequest(request, resolved));
  });
}

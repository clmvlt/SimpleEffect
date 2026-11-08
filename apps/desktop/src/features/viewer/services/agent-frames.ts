import {
  mediaUrl,
  type CoreEndpoint,
  type FontFamily,
  type FrameRenderRequest,
} from '@se/protocol';
import type { FontLoader } from '@se/engine/fonts';
import type { StillRenderer, StillRendererOptions } from '@se/engine/preview';

import type { ProjectMirror } from '../../../shared/project-store';
import { createMediaLocator } from './media-locator';
import { sceneOf } from './viewer-scene';

/**
 * Widest stills drawn from proxies: agents look at small pictures, proxies decode faster and
 * look the same at that size.
 */
const PROXY_MAX_WIDTH = 960;

/** What answering an agent's frame request needs (injected: tests use fakes). */
export interface AgentFrameDeps {
  /** The editor's project, as it is now. */
  readonly mirror: () => ProjectMirror | null;
  /** The core's HTTP endpoint (uploads, media and fonts). */
  readonly endpoint: () => CoreEndpoint | null;
  /** The font catalogue (`null` until listed). */
  readonly families: () => readonly FontFamily[] | null;
  /** A font loader over the catalogue's faces, or `null` where fonts cannot load. */
  readonly fonts: (families: readonly FontFamily[], endpoint: CoreEndpoint) => FontLoader | null;
  /** Starts a still renderer (`@se/engine/preview`, imported lazily). */
  readonly createStills: (options: StillRendererOptions) => Promise<StillRenderer>;
  /** Sends one JPEG to `url` (`PUT`). */
  readonly upload: (url: string, jpeg: Blob) => Promise<void>;
  /** Tells the core (and the waiting agent) why the frames cannot be drawn. */
  readonly reportFailure: (requestId: string, message: string) => Promise<void>;
}

/** Why a request cannot be drawn, for the agent (English, actionable). */
class FrameRequestRefused extends Error {}

/**
 * Draws the frames an AI agent asked for (`renderFramesRequested`, ADR-0028) with the engine the
 * viewer uses, and uploads one JPEG per time to `/media/renders/{requestId}/{index}`; reports a
 * failure to the core instead when the frames cannot be drawn here.
 */
export async function answerFrameRequest(
  request: FrameRenderRequest,
  deps: AgentFrameDeps,
): Promise<void> {
  let stills: StillRenderer | null = null;
  try {
    const { scene, endpoint, assets } = prepare(request, deps);
    const families = deps.families() ?? [];
    stills = await deps.createStills({
      width: request.width,
      height: request.height,
      locator: createMediaLocator(endpoint, assets),
      fonts: (families.length > 0 ? deps.fonts(families, endpoint) : null) ?? undefined,
    });
    const proxy = request.width <= PROXY_MAX_WIDTH;
    for (const [index, time] of request.times.entries()) {
      const still = await stills.render(scene, time, proxy);
      await deps.upload(renderUrl(endpoint, request.requestId, index), still.jpeg);
    }
  } catch (error) {
    await deps.reportFailure(request.requestId, failureMessage(error)).catch(() => undefined);
  } finally {
    stills?.destroy();
  }
}

/** The scene and endpoint of a request, or why it cannot be drawn here. */
function prepare(request: FrameRenderRequest, deps: AgentFrameDeps) {
  const mirror = deps.mirror();
  if (mirror?.projectId !== request.projectId) {
    throw new FrameRequestRefused(
      'the SimpleEffect window is not showing this project: ask the user to open it in the editor',
    );
  }
  const sequence = mirror.sequences[request.sequenceId];
  if (sequence === undefined) {
    throw new FrameRequestRefused('the editor does not know this sequence (it may be deleted)');
  }
  const endpoint = deps.endpoint();
  if (endpoint === null) {
    throw new FrameRequestRefused('the editor lost its connection to the core: try again');
  }
  const scene = sceneOf({
    sequence,
    tracks: mirror.tracks,
    items: mirror.items,
    assets: mirror.assets,
  });
  return { scene, endpoint, assets: mirror.assets };
}

/** Where frame `index` of `requestId` is uploaded (carries the session token: never log it). */
export function renderUrl(endpoint: CoreEndpoint, requestId: string, index: number): string {
  return mediaUrl(endpoint, `/media/renders/${encodeURIComponent(requestId)}/${String(index)}`);
}

function failureMessage(error: unknown): string {
  if (error instanceof FrameRequestRefused) return error.message;
  const detail = error instanceof Error ? error.message : String(error);
  return `the editor could not draw the frames: ${detail}`;
}

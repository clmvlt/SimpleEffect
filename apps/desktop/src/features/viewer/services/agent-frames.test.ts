import type { Flicks, FrameRenderRequest } from '@se/protocol';
import type { StillRenderer, StillRendererOptions } from '@se/engine/preview';
import { describe, expect, it } from 'vitest';

import { mirrorFromSession, type ProjectMirror } from '../../../shared/project-store';
import { IDS, session } from '../../../shared/project-store/testing/session-fixture';
import { answerFrameRequest, renderUrl, type AgentFrameDeps } from './agent-frames';

const endpoint = { url: 'http://127.0.0.1:4100', token: 'secret' };

function request(overrides: Partial<FrameRenderRequest> = {}): FrameRenderRequest {
  return {
    requestId: 'request-1',
    projectId: IDS.project,
    sequenceId: IDS.sequence,
    times: [0 as Flicks, 28_224_000 as Flicks],
    width: 768,
    height: 432,
    ...overrides,
  };
}

/** Deps recording what the service did; the still renderer draws `still <n>`. */
function fakeDeps(mirror: ProjectMirror | null, failRender = false) {
  const uploads: { url: string; body: string }[] = [];
  const failures: { requestId: string; message: string }[] = [];
  const renders: { time: Flicks; proxy: boolean }[] = [];
  const options: StillRendererOptions[] = [];
  let destroyed = 0;
  const stills: StillRenderer = {
    render: (_scene, time, proxy) => {
      renders.push({ time, proxy });
      if (failRender) return Promise.reject(new Error('decoder lost'));
      return Promise.resolve({
        jpeg: new Blob([`still ${String(renders.length)}`]),
        issues: [],
        missingFonts: [],
        templateIssues: [],
      });
    },
    destroy: () => {
      destroyed += 1;
    },
  };
  const deps: AgentFrameDeps = {
    mirror: () => mirror,
    endpoint: () => endpoint,
    families: () => [],
    fonts: () => null,
    templates: () => Promise.resolve(null),
    createStills: (given) => {
      options.push(given);
      return Promise.resolve(stills);
    },
    upload: async (url, jpeg) => {
      uploads.push({ url, body: await jpeg.text() });
    },
    reportFailure: (requestId, message) => {
      failures.push({ requestId, message });
      return Promise.resolve();
    },
  };
  return { deps, uploads, failures, renders, options, destroyed: () => destroyed };
}

describe('answerFrameRequest', () => {
  it('draws every time of the shown project and uploads one JPEG each, in order', async () => {
    const fake = fakeDeps(mirrorFromSession(session()));
    await answerFrameRequest(request(), fake.deps);
    expect(fake.failures).toEqual([]);
    expect(fake.options[0]).toMatchObject({ width: 768, height: 432 });
    expect(fake.renders).toEqual([
      { time: 0, proxy: true },
      { time: 28_224_000, proxy: true },
    ]);
    expect(fake.uploads).toEqual([
      { url: renderUrl(endpoint, 'request-1', 0), body: 'still 1' },
      { url: renderUrl(endpoint, 'request-1', 1), body: 'still 2' },
    ]);
    expect(fake.uploads[0]?.url).toBe('http://127.0.0.1:4100/media/renders/request-1/0?t=secret');
    expect(fake.destroyed()).toBe(1);
  });

  it('draws wide stills from the originals', async () => {
    const fake = fakeDeps(mirrorFromSession(session()));
    await answerFrameRequest(request({ width: 1920, height: 1080 }), fake.deps);
    expect(fake.renders.every((render) => !render.proxy)).toBe(true);
  });

  it('refuses at once when another project, or none, is shown', async () => {
    const other = fakeDeps(mirrorFromSession(session()));
    await answerFrameRequest(request({ projectId: 'project-2' }), other.deps);
    expect(other.failures[0]?.message).toContain('not showing this project');
    expect(other.options).toEqual([]);
    const none = fakeDeps(null);
    await answerFrameRequest(request(), none.deps);
    expect(none.failures).toHaveLength(1);
  });

  it('reports unknown sequences and drawing failures, and frees the renderer', async () => {
    const unknown = fakeDeps(mirrorFromSession(session()));
    await answerFrameRequest(request({ sequenceId: 'nope' }), unknown.deps);
    expect(unknown.failures[0]?.message).toContain('sequence');
    const failing = fakeDeps(mirrorFromSession(session()), true);
    await answerFrameRequest(request(), failing.deps);
    expect(failing.failures).toEqual([
      { requestId: 'request-1', message: 'the editor could not draw the frames: decoder lost' },
    ]);
    expect(failing.uploads).toEqual([]);
    expect(failing.destroyed()).toBe(1);
  });
});

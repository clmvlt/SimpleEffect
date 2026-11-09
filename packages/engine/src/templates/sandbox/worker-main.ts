import type {
  TemplateErrorPlace,
  TemplateParamInput,
  TemplateWorkerReply,
  TemplateWorkerRequest,
} from '../contract';
import type { DrawApiOptions } from './draw-api';
import type { TemplateHelpers } from './helpers';
import type { TextApi, TextApiOptions } from './text-api';

/** What the source builder passes to {@link workerMain}. */
export interface WorkerConfig {
  /** Lines of the Worker's source before the module's first line. */
  readonly lineOffset: number;
}

/** The module's top-level code, wrapped: fills `exports` (CommonJS output of the compiler). */
export type ModuleFactory = (exports: Record<string, unknown>) => void;

/**
 * The template Worker's main function: captures what it needs from the global scope, listens for
 * requests, removes everything else from the scope (`neutralizeGlobals`), runs the module, then
 * answers `fonts`, `images` and `render` requests one at a time.
 *
 * **Self-contained**: serialized with `toString()` into the Worker's source (`sandbox-source.ts`);
 * the other sandbox parts arrive as parameters. No import, no outer variable.
 */
export function workerMain(
  config: WorkerConfig,
  neutralize: (scope: object) => void,
  createHelpers: () => TemplateHelpers,
  createTextApi: (options: TextApiOptions) => TextApi,
  createDrawApi: (options: DrawApiOptions) => object,
  factory: ModuleFactory,
): void {
  const scope = self as unknown as {
    postMessage(message: TemplateWorkerReply, transfer?: Transferable[]): void;
    addEventListener(type: 'message', listener: (event: MessageEvent) => void): void;
    readonly fonts: FontFaceSet;
  };
  const post = scope.postMessage.bind(scope);
  const now = performance.now.bind(performance);
  const Canvas = OffscreenCanvas;
  const Face = FontFace;
  const fontSet = scope.fonts;
  const images = new Map<string, ImageBitmap>();
  const loadedFamilies = new Set<string>();
  let canvas: OffscreenCanvas | null = null;
  let context: OffscreenCanvasRenderingContext2D | null = null;
  let render: ((...args: unknown[]) => unknown) | null = null;

  const place = (error: unknown): TemplateErrorPlace => {
    const message =
      error instanceof Error ? `${error.name}: ${error.message}` : `thrown: ${String(error)}`;
    const stack = error instanceof Error ? (error.stack ?? '') : '';
    for (const match of stack.matchAll(/:(\d+):(\d+)\)?\s*$/gm)) {
      const line = Number(match[1]) - config.lineOffset;
      if (line >= 1) return { message, line, column: Number(match[2]) };
    }
    return { message, line: null, column: null };
  };

  const draw = (request: Extract<TemplateWorkerRequest, { type: 'render' }>) => {
    if (render === null) return;
    if (canvas === null || canvas.width !== request.width || canvas.height !== request.height) {
      canvas = new Canvas(request.width, request.height);
      context = canvas.getContext('2d');
    }
    if (context === null) throw new Error('no 2D context');
    context.reset();
    const scale = request.width / request.designWidth;
    context.setTransform(scale, 0, 0, scale, 0, 0);
    const missingFamilies = new Set<string>();
    const ctx = createDrawApi({
      real: context,
      scale,
      width: request.designWidth,
      height: request.designHeight,
      duration: request.duration,
      images,
      textApi: createTextApi({ real: context, loadedFamilies, missingFamilies }),
      helpers,
    });
    const values: Record<string, TemplateParamInput> = {};
    for (const [name, value] of Object.entries(request.params)) {
      values[name] =
        typeof value === 'object' && value !== null ? Object.freeze({ ...value }) : value;
    }
    const started = now();
    try {
      const result: unknown = render(ctx, request.time, Object.freeze(values));
      if (typeof result === 'object' && result !== null && 'then' in result) {
        throw new TypeError(
          'render must draw synchronously: do not make it async nor return a promise',
        );
      }
      const elapsedMs = now() - started;
      const bitmap = canvas.transferToImageBitmap();
      post(
        {
          type: 'rendered',
          id: request.id,
          bitmap,
          elapsedMs,
          missingFamilies: [...missingFamilies],
        },
        [bitmap],
      );
    } catch (error) {
      post({ type: 'renderFailed', id: request.id, error: place(error) });
    }
  };

  const fonts = (request: Extract<TemplateWorkerRequest, { type: 'fonts' }>) => {
    const faces = request.faces.map((face) => ({
      family: face.family,
      face: new Face(face.family, face.data, {
        weight: face.weight,
        style: face.style,
        ...(face.unicodeRange === null ? {} : { unicodeRange: face.unicodeRange }),
      }),
    }));
    void Promise.allSettled(faces.map((entry) => entry.face.load())).then((results) => {
      results.forEach((result, index) => {
        const entry = faces[index];
        if (result.status !== 'fulfilled' || entry === undefined) return;
        fontSet.add(entry.face);
        loadedFamilies.add(entry.family.toLowerCase());
      });
      post({ type: 'done', id: request.id });
    });
  };

  scope.addEventListener('message', (event: MessageEvent<TemplateWorkerRequest>) => {
    const request = event.data;
    if (request.type === 'render') draw(request);
    else if (request.type === 'fonts') fonts(request);
    else {
      for (const image of request.images) images.set(image.id, image.bitmap);
      post({ type: 'done', id: request.id });
    }
  });
  neutralize(scope);
  const helpers = createHelpers();

  const exported: Record<string, unknown> = {};
  try {
    factory(exported);
  } catch (error) {
    post({ type: 'loadFailed', problems: [place(error)] });
    return;
  }
  const problems: TemplateErrorPlace[] = [];
  const plain = (value: unknown) =>
    typeof value === 'object' && value !== null && !Array.isArray(value);
  if (typeof exported['render'] !== 'function') {
    problems.push({
      message: 'export a function render(ctx, time, params)',
      line: null,
      column: null,
    });
  }
  if (!plain(exported['meta'])) {
    problems.push({
      message: 'export const meta = { name, description, defaultDuration } (an object)',
      line: null,
      column: null,
    });
  }
  if (!plain(exported['params'])) {
    problems.push({
      message: 'export const params = { … } (an object; {} for none)',
      line: null,
      column: null,
    });
  }
  if (problems.length > 0) {
    post({ type: 'loadFailed', problems });
    return;
  }
  try {
    const params = exported['params'] as Record<string, unknown>;
    const declared =
      params['type'] === 'object' && plain(params['properties'])
        ? (params['properties'] as Record<string, unknown>)
        : params;
    const metaJson = JSON.stringify(exported['meta']);
    const paramsJson = JSON.stringify(Object.entries(declared));
    render = exported['render'] as (...args: unknown[]) => unknown;
    post({ type: 'loaded', metaJson, paramsJson });
  } catch (error) {
    post({
      type: 'loadFailed',
      problems: [
        {
          message: `meta and params must be plain data: ${place(error).message}`,
          line: null,
          column: null,
        },
      ],
    });
  }
}

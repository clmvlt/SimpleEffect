import type { Flicks, TemplateCheckOutcome, TemplateProblem } from '@se/protocol';

import { FLICKS_PER_SECOND } from '../time/flicks';
import { compileTemplate } from './compile';
import { TEMPLATE_FRAME_BUDGET_MS, TEMPLATE_WATCHDOG_MS } from './contract';
import { checkDefaults, checkInstants } from './check-plan';
import { differingPixels, encodeJpegBase64 } from './check-pictures';
import type { TemplateFontSource } from './font-faces';
import { templateWorkerSource } from './sandbox-source';
import {
  TemplateFailure,
  TemplateWorker,
  blobWorkerFactory,
  type TemplateFrame,
  type WorkerFactory,
} from './template-worker';
import { DEFAULT_TEMPLATE_FAMILY } from './worker-pool';

/** Options of {@link checkTemplate}. */
export interface TemplateCheckOptions {
  /** The design frame the template draws in (shorter side 1080). */
  readonly width: number;
  readonly height: number;
  /** Size of the pictures returned. */
  readonly pictureWidth: number;
  readonly pictureHeight: number;
  readonly fonts?: TemplateFontSource | undefined;
  readonly createWorker?: WorkerFactory | undefined;
  readonly budgetMs?: number | undefined;
}

type Stage = TemplateProblem['stage'];

/**
 * Checks a template module before it enters a project (prompt 24): compiles it, loads it in its
 * sandbox, renders the start, entrance, middle, exit and end with the default values at the
 * design size twice — a warm-up pass (fonts given, caches filled), then a timed pass where each
 * frame must fit the budget —, compares both passes pixel by pixel (determinism), and returns
 * three pictures (entrance, middle, exit) with the module's `meta` and `params`. Every problem
 * comes back with its stage, line and instant.
 */
export async function checkTemplate(
  source: string,
  options: TemplateCheckOptions,
): Promise<TemplateCheckOutcome> {
  const compiled = compileTemplate(source);
  if (!compiled.ok) {
    return failed(
      compiled.problems.map((problem) => ({ ...problem, stage: 'compile' as const, time: null })),
    );
  }
  let worker: TemplateWorker;
  try {
    worker = await TemplateWorker.start(
      templateWorkerSource(compiled.compiled),
      options.createWorker ?? blobWorkerFactory,
    );
  } catch (error) {
    return failed(problemsOf(error, 'exports', null));
  }
  try {
    return await renderChecks(worker, options);
  } finally {
    worker.terminate();
  }
}

async function renderChecks(
  worker: TemplateWorker,
  options: TemplateCheckOptions,
): Promise<TemplateCheckOutcome> {
  const { metaJson, paramsJson } = worker.declarations;
  const defaults = checkDefaults(metaJson, paramsJson);
  await provideFonts(worker, options.fonts, [DEFAULT_TEMPLATE_FAMILY, ...defaults.families]);
  const budget = options.budgetMs ?? TEMPLATE_FRAME_BUDGET_MS;
  const render = (time: number) =>
    worker.render(
      {
        time,
        duration: defaults.duration,
        width: options.width,
        height: options.height,
        designWidth: options.width,
        designHeight: options.height,
        params: defaults.params,
      },
      TEMPLATE_WATCHDOG_MS,
    );
  const instants = checkInstants(defaults.duration);
  let current = 0;
  const renderAt = (time: number) => {
    current = time;
    return render(time);
  };
  const warm = new Map<number, TemplateFrame>();
  const timed = new Map<number, TemplateFrame>();
  try {
    await renderPass(renderAt, instants.timed, warm);
    const missing = [...new Set([...warm.values()].flatMap((frame) => frame.missingFamilies))];
    if (missing.length > 0) {
      await provideFonts(worker, options.fonts, missing);
      closeAll(warm);
      warm.clear();
      await renderPass(renderAt, instants.timed, warm);
    }
    await renderPass(renderAt, instants.timed, timed);
  } catch (error) {
    closeAll(warm);
    closeAll(timed);
    return failed(problemsOf(error, 'render', current));
  }
  const problems = [
    ...[...timed]
      .filter(([, frame]) => frame.elapsedMs > budget)
      .map(([time, frame]) => slowProblem(time, frame.elapsedMs, budget)),
    ...determinismProblems(warm, timed),
  ];
  closeAll(warm);
  if (problems.length > 0) {
    closeAll(timed);
    return failed(problems);
  }
  const pictures = await Promise.all(
    instants.pictures.map(async (time) => ({
      time: toFlicks(time),
      jpegBase64: await encodeJpegBase64(
        timed.get(time)?.bitmap ?? null,
        options.pictureWidth,
        options.pictureHeight,
      ),
    })),
  );
  const slowest = Math.max(0, ...[...timed.values()].map((frame) => frame.elapsedMs));
  closeAll(timed);
  return { kind: 'passed', metaJson, paramsJson, frames: pictures, slowestFrameMs: slowest };
}

/** Renders each instant once, in order, into `frames`. */
async function renderPass(
  render: (time: number) => Promise<TemplateFrame>,
  instants: readonly number[],
  frames: Map<number, TemplateFrame>,
): Promise<void> {
  for (const time of instants) frames.set(time, await render(time));
}

/**
 * Compares the warm-up pass with the timed pass, instant by instant: a template keeping state
 * between calls (or reading anything but its inputs) draws something else the second time.
 */
function determinismProblems(
  warm: ReadonlyMap<number, TemplateFrame>,
  timed: ReadonlyMap<number, TemplateFrame>,
): TemplateProblem[] {
  const problems: TemplateProblem[] = [];
  for (const [time, frame] of timed) {
    const first = warm.get(time);
    const differing = first === undefined ? 0 : differingPixels(first.bitmap, frame.bitmap);
    if (differing === 0) continue;
    problems.push({
      stage: 'determinism',
      message:
        `rendering ${time.toFixed(3)} s twice gave different pictures (${String(differing)} ` +
        'pixels differ): render must depend only on ctx, time and params — no state kept ' +
        'between calls (counters, caches of drawn things), no Math.random (use ctx.random(seed))',
      line: null,
      column: null,
      time: toFlicks(time),
    });
    break;
  }
  return problems;
}

async function provideFonts(
  worker: TemplateWorker,
  fonts: TemplateFontSource | undefined,
  families: readonly string[],
): Promise<void> {
  if (fonts === undefined || families.length === 0) return;
  const unique = [...new Set(families.map((family) => family.trim()).filter((f) => f !== ''))];
  const faces = (await Promise.all(unique.map((family) => fonts(family)))).flat();
  if (faces.length > 0) await worker.sendFonts(faces, TEMPLATE_WATCHDOG_MS);
}

function slowProblem(time: number, elapsedMs: number, budget: number): TemplateProblem {
  return {
    stage: 'budget',
    message:
      `the frame at ${time.toFixed(3)} s took ${elapsedMs.toFixed(1)} ms; the budget is ` +
      `${String(budget)} ms per frame: draw less (fewer shadows and blurs, no per-pixel loops, ` +
      'no large repeated text measurements)',
    line: null,
    column: null,
    time: toFlicks(time),
  };
}

function problemsOf(error: unknown, stage: Stage, time: number | null): TemplateProblem[] {
  const failure =
    error instanceof TemplateFailure
      ? error
      : new TemplateFailure('exception', [{ message: String(error), line: null, column: null }]);
  const resolved: Stage = failure.kind === 'timeout' ? 'budget' : stage;
  return failure.places.map((place) => ({
    stage: resolved,
    message: place.message,
    line: place.line,
    column: place.column,
    time: time === null ? null : toFlicks(time),
  }));
}

function failed(problems: TemplateProblem[]): TemplateCheckOutcome {
  return { kind: 'failed', problems };
}

function closeAll(frames: ReadonlyMap<number, TemplateFrame>): void {
  for (const frame of frames.values()) frame.bitmap.close();
}

function toFlicks(seconds: number): Flicks {
  return Math.round(seconds * FLICKS_PER_SECOND) as Flicks;
}

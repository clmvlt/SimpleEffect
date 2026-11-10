import type { CatalogFace } from '@se/engine/fonts';
import type {
  ExportScene,
  ExportSession,
  ExportSessionOptions,
  ExportSessionResult,
} from '@se/engine/export';
import type {
  CoreEndpoint,
  ExportId,
  ExportRenderRequest,
  ProjectSnapshot,
  SequenceEntity,
} from '@se/protocol';

import { sceneOf } from '../../viewer';

/** Least time between two progress reports to the core. */
export const PROGRESS_INTERVAL_MS = 250;

/** What rendering an export needs (injected: tests use fakes). */
export interface ExportRenderDeps {
  /** The core's HTTP endpoint (`null` while disconnected). */
  readonly endpoint: () => CoreEndpoint | null;
  /** The font catalogue's faces on `endpoint` (texts and motion templates). */
  readonly fonts: (endpoint: CoreEndpoint) => Promise<CatalogFace[]>;
  /** Starts the engine's export session (`@se/engine/export`, imported lazily). */
  readonly startSession: (options: ExportSessionOptions) => Promise<ExportSession>;
  readonly reportProgress: (exportId: ExportId, done: number, total: number) => Promise<void>;
  readonly finish: (exportId: ExportId) => Promise<void>;
  readonly fail: (exportId: ExportId, message: string) => Promise<void>;
  /** Told what a finished export could not draw (missing media, fonts, failing templates). */
  readonly onIssues: (exportId: ExportId, issues: ExportSessionResult) => void;
  readonly now: () => number;
}

/** A started render: cancel it when the core says the export was cancelled. */
export interface ExportRender {
  readonly done: Promise<void>;
  cancel(): void;
}

function byId<T extends { readonly id: string }>(entities: readonly T[]): Record<string, T> {
  return Object.fromEntries(entities.map((entity) => [entity.id, entity]));
}

function snapshotSequence(snapshot: ProjectSnapshot, sequenceId: string): SequenceEntity {
  const sequence = snapshot.sequences.find((candidate) => candidate.id === sequenceId);
  if (sequence === undefined) throw new Error('the export snapshot lacks its sequence');
  return sequence;
}

/** The scene of `sequenceId` in an export's snapshot, as the viewer would play it. */
export function snapshotScene(snapshot: ProjectSnapshot, sequenceId: string): ExportScene {
  const sequence = snapshotSequence(snapshot, sequenceId);
  return sceneOf({
    sequence,
    tracks: byId(snapshot.tracks),
    items: byId(snapshot.items),
    assets: byId(snapshot.assets),
    captionStyles: snapshot.project.captionStyles,
    templates: snapshot.project.templates,
  });
}

/** Whether `error` means the export was stopped (cancelled, or ended by the core). */
function isStopped(error: unknown): boolean {
  return error instanceof Error && error.name === 'ExportStoppedError';
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

/** Reports progress at most every {@link PROGRESS_INTERVAL_MS}, the last step always. */
function progressReporter(request: ExportRenderRequest, deps: ExportRenderDeps) {
  let last = Number.NEGATIVE_INFINITY;
  return (done: number, total: number) => {
    const now = deps.now();
    if (done < total && now - last < PROGRESS_INTERVAL_MS) return;
    last = now;
    deps.reportProgress(request.exportId, done, total).catch(() => undefined);
  };
}

async function run(
  request: ExportRenderRequest,
  deps: ExportRenderDeps,
  started: (session: ExportSession) => boolean,
): Promise<void> {
  const endpoint = deps.endpoint();
  if (endpoint === null) throw new Error('the editor lost its connection to the core');
  const scene = snapshotScene(request.snapshot, request.sequenceId);
  const session = await deps.startSession({
    job: {
      exportId: request.exportId,
      endpoint,
      scene,
      fonts: await deps.fonts(endpoint),
      settings: request.settings,
      pipeline: request.pipeline,
      start: request.start,
      end: request.end,
      frameRate: snapshotSequence(request.snapshot, request.sequenceId).format.frameRate,
    },
    onProgress: progressReporter(request, deps),
  });
  if (!started(session)) return;
  const issues = await session.done;
  await deps.finish(request.exportId);
  deps.onIssues(request.exportId, issues);
}

/**
 * Renders an export the core asked for (`exportRenderRequested`, ADR-0038) from its snapshot with
 * the engine's export session: progress is reported (throttled), then the outcome — finished, or
 * failed with the reason. A stopped export reports nothing: the core already knows.
 */
export function startExportRender(
  request: ExportRenderRequest,
  deps: ExportRenderDeps,
): ExportRender {
  let session: ExportSession | null = null;
  let cancelled = false;
  const started = (value: ExportSession) => {
    session = value;
    if (cancelled) {
      value.done.catch(() => undefined);
      value.cancel();
    }
    return !cancelled;
  };
  const done = run(request, deps, started).catch(async (error: unknown) => {
    if (cancelled || isStopped(error)) return;
    await deps.fail(request.exportId, messageOf(error)).catch(() => undefined);
  });
  return {
    done,
    cancel: () => {
      cancelled = true;
      session?.cancel();
    },
  };
}

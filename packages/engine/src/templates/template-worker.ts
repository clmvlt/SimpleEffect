import {
  TEMPLATE_LOAD_WATCHDOG_MS,
  type TemplateErrorPlace,
  type TemplateFontFace,
  type TemplateParamInput,
  type TemplateWorkerReply,
  type TemplateWorkerRequest,
} from './contract';
import type { WorkerSource } from './sandbox-source';

/** Why a template Worker could not render. */
export type TemplateFailureKind =
  /** The module threw, or lacks an export. */
  | 'exception'
  /** It ran longer than allowed (an endless loop?): the Worker was killed. */
  | 'timeout'
  /** The Worker died or could not start (a syntax error the compiler let through, a crash). */
  | 'crashed';

/** A failure of a template Worker, with where it happened in the module. */
export class TemplateFailure extends Error {
  constructor(
    readonly kind: TemplateFailureKind,
    readonly places: readonly TemplateErrorPlace[],
  ) {
    super(places.map((place) => place.message).join('; '));
    this.name = 'TemplateFailure';
  }
}

/** A frame rendered by a template. */
export interface TemplateFrame {
  readonly bitmap: ImageBitmap;
  readonly elapsedMs: number;
  readonly missingFamilies: readonly string[];
}

/** One frame to render, in the Worker's terms. */
export interface TemplateFrameRequest {
  readonly time: number;
  readonly duration: number;
  readonly width: number;
  readonly height: number;
  readonly designWidth: number;
  readonly designHeight: number;
  readonly params: Readonly<Record<string, TemplateParamInput>>;
}

/** The exports the module declared. */
export interface TemplateDeclarations {
  readonly metaJson: string;
  readonly paramsJson: string;
}

interface Pending {
  resolve(reply: TemplateWorkerReply): void;
  reject(failure: TemplateFailure): void;
}

/** Creates the Worker of a source (a blob Worker in the app; injectable for tests). */
export type WorkerFactory = (source: string) => Worker;

/** The Worker of a source, created from a blob whose URL is revoked once the Worker started. */
export const blobWorkerFactory: WorkerFactory = (source) => {
  const url = URL.createObjectURL(new Blob([source], { type: 'text/javascript' }));
  try {
    return new Worker(url, { name: 'motion-template' });
  } finally {
    setTimeout(() => {
      URL.revokeObjectURL(url);
    }, 10_000);
  }
};

/**
 * The main thread's side of one template Worker: loads the module, sends fonts and images, and
 * renders frames one at a time, each under a watchdog that kills the Worker when it overruns.
 * A killed or crashed Worker refuses everything afterwards (`alive` is false): start another.
 */
export class TemplateWorker {
  private readonly pending = new Map<number, Pending>();
  private nextId = 1;
  private dead: TemplateFailure | null = null;
  private queue: Promise<unknown> = Promise.resolve();

  private constructor(
    private readonly worker: Worker,
    private readonly lineOffset: number,
    readonly declarations: TemplateDeclarations,
  ) {
    worker.onmessage = (event: MessageEvent<TemplateWorkerReply>) => {
      this.receive(event.data);
    };
    worker.onerror = (event) => {
      event.preventDefault();
      this.kill(new TemplateFailure('crashed', [this.errorPlace(event)]));
    };
  }

  /**
   * Starts a Worker for `source` and waits for its module to load.
   *
   * @throws {TemplateFailure} when the module throws, lacks an export, never finishes loading
   * or the Worker cannot start.
   */
  static start(
    source: WorkerSource,
    createWorker: WorkerFactory = blobWorkerFactory,
    watchdogMs: number = TEMPLATE_LOAD_WATCHDOG_MS,
  ): Promise<TemplateWorker> {
    return new Promise((resolve, reject) => {
      const worker = createWorker(source.source);
      const fail = (failure: TemplateFailure) => {
        clearTimeout(timer);
        worker.terminate();
        reject(failure);
      };
      const timer = setTimeout(() => {
        fail(
          new TemplateFailure('timeout', [
            {
              message: `the module did not finish loading within ${String(watchdogMs)} ms (an endless loop at the top level?)`,
              line: null,
              column: null,
            },
          ]),
        );
      }, watchdogMs);
      worker.onerror = (event) => {
        event.preventDefault();
        fail(new TemplateFailure('crashed', [errorEventPlace(event, source.lineOffset)]));
      };
      worker.onmessage = (event: MessageEvent<TemplateWorkerReply>) => {
        const reply = event.data;
        if (reply.type === 'loadFailed') fail(new TemplateFailure('exception', reply.problems));
        else if (reply.type === 'loaded') {
          clearTimeout(timer);
          resolve(new TemplateWorker(worker, source.lineOffset, reply));
        }
      };
    });
  }

  /** Whether the Worker still runs. */
  get alive(): boolean {
    return this.dead === null;
  }

  /** Gives the Worker font faces (it loads them before answering). */
  sendFonts(faces: readonly TemplateFontFace[], watchdogMs: number): Promise<void> {
    return this.enqueue(async () => {
      await this.request({ type: 'fonts', id: 0, faces }, watchdogMs);
    });
  }

  /** Gives the Worker pictures by handle id (transferred: they become unusable here). */
  sendImages(
    images: readonly { readonly id: string; readonly bitmap: ImageBitmap }[],
    watchdogMs: number,
  ): Promise<void> {
    return this.enqueue(async () => {
      await this.request(
        { type: 'images', id: 0, images },
        watchdogMs,
        images.map((image) => image.bitmap),
      );
    });
  }

  /**
   * Renders one frame; the Worker is killed when it takes longer than `watchdogMs`.
   *
   * @throws {TemplateFailure}
   */
  render(request: TemplateFrameRequest, watchdogMs: number): Promise<TemplateFrame> {
    return this.enqueue(async () => {
      const reply = await this.request({ type: 'render', id: 0, ...request }, watchdogMs);
      if (reply.type === 'renderFailed') throw new TemplateFailure('exception', [reply.error]);
      if (reply.type !== 'rendered') throw new TemplateFailure('crashed', [unexpected()]);
      return {
        bitmap: reply.bitmap,
        elapsedMs: reply.elapsedMs,
        missingFamilies: reply.missingFamilies,
      };
    });
  }

  /** Stops the Worker; pending requests fail. */
  terminate(): void {
    this.kill(
      new TemplateFailure('crashed', [
        { message: 'the template was unloaded', line: null, column: null },
      ]),
    );
  }

  private enqueue<T>(work: () => Promise<T>): Promise<T> {
    const next = this.queue.then(work, work);
    this.queue = next.catch(() => undefined);
    return next;
  }

  private request(
    message: TemplateWorkerRequest,
    watchdogMs: number,
    transfer: Transferable[] = [],
  ): Promise<TemplateWorkerReply> {
    if (this.dead !== null) return Promise.reject(this.dead);
    const id = this.nextId++;
    return new Promise((resolve, reject) => {
      const timer = setTimeout(() => {
        this.kill(
          new TemplateFailure('timeout', [
            {
              message: `the frame took longer than ${String(watchdogMs)} ms (an endless loop?): the template was stopped`,
              line: null,
              column: null,
            },
          ]),
        );
      }, watchdogMs);
      this.pending.set(id, {
        resolve: (reply) => {
          clearTimeout(timer);
          resolve(reply);
        },
        reject: (failure) => {
          clearTimeout(timer);
          reject(failure);
        },
      });
      this.worker.postMessage({ ...message, id }, transfer);
    });
  }

  private receive(reply: TemplateWorkerReply): void {
    if (!('id' in reply)) return;
    const pending = this.pending.get(reply.id);
    if (pending === undefined) {
      if (reply.type === 'rendered') reply.bitmap.close();
      return;
    }
    this.pending.delete(reply.id);
    pending.resolve(reply);
  }

  private kill(failure: TemplateFailure): void {
    if (this.dead !== null) return;
    this.dead = failure;
    this.worker.terminate();
    for (const pending of this.pending.values()) pending.reject(failure);
    this.pending.clear();
  }

  private errorPlace(event: ErrorEvent): TemplateErrorPlace {
    return errorEventPlace(event, this.lineOffset);
  }
}

function errorEventPlace(event: ErrorEvent, lineOffset: number): TemplateErrorPlace {
  const line = typeof event.lineno === 'number' ? event.lineno - lineOffset : 0;
  return {
    message: event.message === '' ? 'the template could not start' : event.message,
    line: line >= 1 ? line : null,
    column: line >= 1 && typeof event.colno === 'number' ? event.colno : null,
  };
}

function unexpected(): TemplateErrorPlace {
  return { message: 'the template answered unexpectedly', line: null, column: null };
}

import type { PlayableSequence } from '../audio/audio-clips';
import type { AudioReaderFactory } from '../audio/audio-source';
import type { RenderScene } from '../preview/frame-renderer';
import { ExportAudioMixer } from './audio-mixer';
import type { ExportWorkerJob, FromExportWorker, ToExportWorker } from './export-messages';
import { ExportStoppedError, exportMediaLocator } from './export-sink';

/** A scene an export draws and mixes (the viewer's playback scene fits). */
export interface ExportScene extends RenderScene {
  readonly sequence: RenderScene['sequence'] & PlayableSequence;
}

/** What a finished export could not draw as asked. */
export interface ExportSessionResult {
  readonly missingAssets: readonly string[];
  readonly missingFonts: readonly string[];
  readonly failedTemplates: readonly string[];
}

/** Options of {@link startExportSession}. */
export interface ExportSessionOptions {
  readonly job: ExportWorkerJob & { readonly scene: ExportScene };
  readonly onProgress: (done: number, total: number) => void;
  /** Starts the export Worker (tests may give their own). */
  readonly createWorker?: () => Worker;
  readonly createReader?: AudioReaderFactory;
}

/** A running export. */
export interface ExportSession {
  /** Resolves when the file is written; rejects with the reason, or `ExportStoppedError`. */
  readonly done: Promise<ExportSessionResult>;
  /** Stops at once: the Worker is terminated (decoders, GPU context, uploads with it). */
  cancel(): void;
}

/** The export Worker, bundled by Vite as a module Worker. */
export function createExportWorker(): Worker {
  return new Worker(new URL('./export-worker.ts', import.meta.url), {
    type: 'module',
    name: 'export',
  });
}

function messageOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

function mixerOf(options: ExportSessionOptions): ExportAudioMixer | null {
  const { job } = options;
  if (job.settings.audio === null) return null;
  return new ExportAudioMixer({
    sequence: job.scene.sequence,
    assets: job.scene.assets,
    locator: exportMediaLocator(job.endpoint, job.exportId),
    start: job.start,
    end: job.end,
    ...(options.createReader === undefined ? {} : { createReader: options.createReader }),
  });
}

/** The page side of one export: answers the Worker, settles `done` once. */
class WorkerExportSession implements ExportSession {
  readonly done: Promise<ExportSessionResult>;
  private readonly worker: Worker;
  private readonly mixer: ExportAudioMixer | null;
  private settled = false;
  private resolve: (result: ExportSessionResult) => void = () => undefined;
  private reject: (error: Error) => void = () => undefined;

  constructor(private readonly options: ExportSessionOptions) {
    this.worker = (options.createWorker ?? createExportWorker)();
    this.mixer = mixerOf(options);
    this.done = new Promise((resolve, reject) => {
      this.resolve = resolve;
      this.reject = reject;
    });
    this.worker.onmessage = (event: MessageEvent<FromExportWorker>) => {
      this.receive(event.data);
    };
    this.worker.onerror = (event: ErrorEvent) => {
      event.preventDefault();
      const detail = event.message === '' ? 'the renderer crashed' : event.message;
      this.fail(new Error(`the export stopped unexpectedly: ${detail}`));
    };
    this.send({ kind: 'start', job: options.job });
  }

  cancel(): void {
    this.fail(new ExportStoppedError());
  }

  private receive(message: FromExportWorker): void {
    if (this.settled) return;
    switch (message.kind) {
      case 'needSound':
        this.mixFor(message.index);
        return;
      case 'progress':
        this.options.onProgress(message.done, message.total);
        return;
      case 'finished': {
        this.settle();
        const { missingAssets, missingFonts, failedTemplates } = message;
        this.resolve({ missingAssets, missingFonts, failedTemplates });
        return;
      }
      case 'failed':
        this.fail(message.stopped ? new ExportStoppedError() : new Error(message.message));
        return;
    }
  }

  private mixFor(index: number): void {
    if (this.mixer === null) {
      this.send({ kind: 'soundFailed', index, message: 'this export has no sound' });
      return;
    }
    this.mixer.mix(index).then(
      ({ channels }) => {
        const buffers = channels.map((data) => data.buffer);
        if (!this.settled) this.send({ kind: 'sound', index, channels }, buffers);
      },
      (error: unknown) => {
        if (!this.settled) this.send({ kind: 'soundFailed', index, message: messageOf(error) });
      },
    );
  }

  private send(message: ToExportWorker, transfer: Transferable[] = []): void {
    this.worker.postMessage(message, transfer);
  }

  private fail(error: Error): void {
    if (this.settled) return;
    this.settle();
    this.reject(error);
  }

  private settle(): void {
    this.settled = true;
    this.worker.terminate();
    this.mixer?.destroy();
  }
}

/**
 * Runs an export (ADR-0038): the Worker draws, encodes and uploads; this page mixes each block
 * of sound it asks for ({@link ExportAudioMixer}: Workers have no `OfflineAudioContext`) and
 * transfers it. Progress goes to `onProgress`; the outcome to `done`.
 */
export function startExportSession(options: ExportSessionOptions): ExportSession {
  return new WorkerExportSession(options);
}

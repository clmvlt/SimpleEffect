import type { CoreEndpoint, ExportId } from '@se/protocol';

import type { FromExportWorker, ToExportWorker, ExportWorkerJob } from './export-messages';
import { createExportRenderer, type ExportRenderer } from './export-renderer';
import {
  ExportStoppedError,
  exportMediaLocator,
  httpExportSink,
  type ExportSink,
} from './export-sink';
import { renderExport } from './render-export';

/** Where the runner's messages to the page go (the Worker's `postMessage`). */
export type ExportMessagePoster = (message: FromExportWorker) => void;
/** Creates the sink of an export (the core's upload routes; memory in tests). */
export type ExportSinkFactory = (endpoint: CoreEndpoint, exportId: ExportId) => ExportSink;

interface Waiting {
  readonly resolve: (channels: Float32Array[]) => void;
  readonly reject: (error: Error) => void;
}

/** `message` without URLs: media and upload URLs carry the session token. */
export function withoutUrls(message: string): string {
  return message.replace(/\b(?:https?|file|blob):\/\/\S+/g, '<url>');
}

/**
 * The export Worker's logic (ADR-0038): renders the job it is given with {@link renderExport},
 * asks the page for each block of mixed sound, uploads to the core and reports progress and the
 * outcome. One job per runner; the page stops an export by terminating the Worker.
 */
export class ExportJobRunner {
  private readonly waiting = new Map<number, Waiting>();
  private started = false;

  constructor(
    private readonly post: ExportMessagePoster,
    private readonly createSink: ExportSinkFactory = httpExportSink,
  ) {}

  /** Handles a message from the page. */
  receive(message: ToExportWorker): void {
    switch (message.kind) {
      case 'start':
        if (this.started) return;
        this.started = true;
        void this.run(message.job);
        return;
      case 'sound':
        this.waiting.get(message.index)?.resolve(message.channels);
        this.waiting.delete(message.index);
        return;
      case 'soundFailed':
        this.waiting.get(message.index)?.reject(new Error(message.message));
        this.waiting.delete(message.index);
        return;
    }
  }

  private sound(index: number): Promise<Float32Array[]> {
    return new Promise((resolve, reject) => {
      this.waiting.set(index, { resolve, reject });
      this.post({ kind: 'needSound', index });
    });
  }

  private async run(job: ExportWorkerJob): Promise<void> {
    const locator = exportMediaLocator(job.endpoint, job.exportId);
    let renderer: ExportRenderer | null = null;
    try {
      renderer = await createExportRenderer(job, locator);
      const { drawer } = renderer;
      const sink = this.createSink(job.endpoint, job.exportId);
      const thumbnails = renderer;
      await renderExport(job, {
        canvas: renderer.canvas,
        draw: (time) => (drawer === null ? Promise.resolve() : drawer.draw(time)),
        readRgba: () => thumbnails.readRgba(),
        sink,
        sound: (index) => this.sound(index),
        thumbnail: async () => {
          const jpeg = await thumbnails.thumbnail();
          if (jpeg !== null) await sink.thumbnail(jpeg);
        },
        onProgress: (done, total) => {
          this.post({ kind: 'progress', done, total });
        },
        signal: new AbortController().signal,
      });
      const issues = drawer?.issues();
      this.post({
        kind: 'finished',
        missingAssets: [...(issues?.assets ?? [])],
        missingFonts: [...(issues?.fonts ?? [])],
        failedTemplates: [...(issues?.templates ?? [])],
      });
    } catch (error) {
      this.post({
        kind: 'failed',
        message: withoutUrls(error instanceof Error ? error.message : String(error)),
        stopped: error instanceof ExportStoppedError,
      });
    } finally {
      renderer?.destroy();
    }
  }
}

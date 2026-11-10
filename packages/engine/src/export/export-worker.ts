// Pixi generates shader code with `new Function` unless this polyfill is loaded first.
import 'pixi.js/unsafe-eval';

import { DOMAdapter, WebWorkerAdapter } from 'pixi.js';

import { ExportJobRunner } from './export-job-runner';
import type { ToExportWorker } from './export-messages';

/*
 * The export Worker (ADR-0038): renders one export off the page's thread, so it neither blocks
 * editing nor depends on the window being visible. PixiJS draws through `OffscreenCanvas` here.
 */
DOMAdapter.set(WebWorkerAdapter);

const runner = new ExportJobRunner((message) => {
  self.postMessage(message);
});

self.onmessage = (event: MessageEvent<ToExportWorker>) => {
  runner.receive(event.data);
};

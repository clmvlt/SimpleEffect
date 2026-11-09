import type { CompiledTemplate } from './compile';
import { createDrawApi } from './sandbox/draw-api';
import { createHelpers } from './sandbox/helpers';
import { neutralizeGlobals } from './sandbox/neutralize';
import { createTextApi } from './sandbox/text-api';
import { workerMain } from './sandbox/worker-main';

/** The source of a template Worker and where the module's lines start in it. */
export interface WorkerSource {
  readonly source: string;
  /** Lines before the module's first line (error lines are shifted back by it). */
  readonly lineOffset: number;
}

/**
 * The source of the Worker running `compiled` (one Worker per template version, created from a
 * blob): the sandbox's functions serialized as expressions, then the module wrapped in a function
 * receiving `exports`.
 *
 * Nothing is declared at the script's top level: the module's code can see only the (neutralized)
 * global scope, never the runtime's own variables. The wrapper's parameters shadow the global
 * object's names as a second line of defence. A blob Worker inherits the page's CSP: no
 * `eval`, connections to the local core only — and the network objects are removed anyway.
 */
export function templateWorkerSource(compiled: CompiledTemplate): WorkerSource {
  const prefix = (lineOffset: number) =>
    [
      '"use strict";',
      `(${workerMain.toString()})(`,
      `${JSON.stringify({ lineOffset })},`,
      `(${neutralizeGlobals.toString()}),`,
      `(${createHelpers.toString()}),`,
      `(${createTextApi.toString()}),`,
      `(${createDrawApi.toString()}),`,
      'function (exports, globalThis, self, window, postMessage, close, importScripts) {',
      '"use strict";',
      '',
    ].join('\n');
  const lineOffset = prefix(0).split('\n').length - 1;
  return {
    source: `${prefix(lineOffset)}${compiled.code}\n});\n`,
    lineOffset,
  };
}

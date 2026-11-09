/**
 * Removes from a template Worker's global scope everything a template must not reach: the
 * network, timers, clocks and randomness, storage, other workers, the Worker's own messaging,
 * code generation. Each removed name becomes a getter throwing an error that names it, so the
 * agent learns what it used. Built-in prototypes are frozen afterwards.
 *
 * **Self-contained**: serialized with `toString()` into the Worker's source (`sandbox-source.ts`).
 * It may use only its parameters and JavaScript built-ins — no import, no outer variable.
 */
export function neutralizeGlobals(scope: object): void {
  const blocked = [
    // Network and other processes.
    'fetch',
    'XMLHttpRequest',
    'WebSocket',
    'WebSocketStream',
    'WebTransport',
    'EventSource',
    'importScripts',
    'Worker',
    'SharedWorker',
    'BroadcastChannel',
    'MessageChannel',
    'MessagePort',
    'Request',
    'Response',
    'Headers',
    'FormData',
    'navigator',
    'location',
    // Storage.
    'indexedDB',
    'caches',
    'CacheStorage',
    'Blob',
    'File',
    'FileReader',
    'FileReaderSync',
    'URL',
    'URLSearchParams',
    // Time, randomness and nondeterminism.
    'setTimeout',
    'setInterval',
    'clearTimeout',
    'clearInterval',
    'queueMicrotask',
    'requestAnimationFrame',
    'cancelAnimationFrame',
    'performance',
    'crypto',
    'Crypto',
    'SubtleCrypto',
    'WeakRef',
    'FinalizationRegistry',
    'SharedArrayBuffer',
    'Atomics',
    // Pictures and fonts come through the drawing API only.
    'OffscreenCanvas',
    'OffscreenCanvasRenderingContext2D',
    'createImageBitmap',
    'ImageBitmap',
    'ImageData',
    'FontFace',
    'fonts',
    // Code generation and the Worker's own channel.
    'eval',
    'Function',
    'WebAssembly',
    'postMessage',
    'close',
    'onmessage',
    'onerror',
    'addEventListener',
    'removeEventListener',
    'dispatchEvent',
    'self',
    'globalThis',
  ];
  const refuse = (name: string) => () => {
    throw new ReferenceError(
      `${name} is not available in motion templates: they cannot reach the network, the clock, ` +
        'timers, storage or randomness. Draw with ctx only and derive everything from time and ' +
        'params (ctx.random(seed) for deterministic randomness).',
    );
  };
  const chain: object[] = [];
  for (
    let level: object | null = scope;
    level !== null;
    level = Object.getPrototypeOf(level) as object | null
  ) {
    chain.push(level);
  }
  for (const name of blocked) {
    for (const level of chain) {
      if (Object.prototype.hasOwnProperty.call(level, name)) {
        try {
          Reflect.deleteProperty(level, name);
        } catch {
          // A property that cannot be deleted is shadowed below.
        }
      }
    }
    try {
      Object.defineProperty(scope, name, { get: refuse(name), configurable: false });
    } catch {
      // Already non-configurable: left as it is (the CSP still blocks the network).
    }
  }
  const blockedConstructor = refuse('the Function constructor');
  const functionPrototypes = [
    Object.getPrototypeOf(function () {}),
    Object.getPrototypeOf(async function () {}),
    Object.getPrototypeOf(function* () {}),
    Object.getPrototypeOf(async function* () {}),
  ] as object[];
  for (const prototype of functionPrototypes) {
    Object.defineProperty(prototype, 'constructor', {
      value: blockedConstructor,
      writable: false,
      configurable: false,
    });
  }
  const notAllowed = (name: string) => () => {
    throw new ReferenceError(
      `${name} is not available in motion templates: a template must draw the same picture at ` +
        'the same time on every run. Use the time argument and ctx.random(seed).',
    );
  };
  Object.defineProperty(Math, 'random', { value: notAllowed('Math.random'), writable: false });
  Object.defineProperty(scope, 'Date', { get: notAllowed('Date'), configurable: false });
  Object.defineProperty(Error, 'prepareStackTrace', {
    value: undefined,
    writable: false,
    configurable: false,
  });
  for (const frozen of [
    Math,
    JSON,
    Reflect,
    Object,
    Array,
    Promise,
    String,
    Number,
    Object.prototype,
    Array.prototype,
    functionPrototypes[0] ?? Object.prototype,
    Promise.prototype,
    String.prototype,
    Number.prototype,
  ]) {
    Object.freeze(frozen);
  }
}

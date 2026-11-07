// Test environment of the desktop app. Renderer component tests run in jsdom (docblock
// `@vitest-environment jsdom`); Node tests share this file, so DOM setup is conditional.
import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

if (typeof window !== 'undefined') {
  // Vitest runs without globals, so Testing Library cannot register its own cleanup.
  afterEach(() => {
    cleanup();
  });

  /** No-op ResizeObserver: jsdom has no layout, so there is never anything to report. */
  class ResizeObserverStub implements ResizeObserver {
    observe(): void {}
    unobserve(): void {}
    disconnect(): void {}
  }
  if (!('ResizeObserver' in globalThis)) {
    Object.defineProperty(globalThis, 'ResizeObserver', { value: ResizeObserverStub });
  }
  // Pointer drags (timeline) capture the pointer; jsdom does not implement capture.
  if (!('setPointerCapture' in Element.prototype)) {
    Object.assign(Element.prototype, {
      setPointerCapture: () => undefined,
      releasePointerCapture: () => undefined,
      hasPointerCapture: () => false,
    });
  }
  // sonner reads the colour scheme; jsdom has no `matchMedia`.
  if (!('matchMedia' in window)) {
    Object.defineProperty(window, 'matchMedia', {
      value: (query: string) => ({
        matches: false,
        media: query,
        onchange: null,
        addEventListener: () => undefined,
        removeEventListener: () => undefined,
        dispatchEvent: () => false,
      }),
    });
  }
}

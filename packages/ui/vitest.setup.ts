// Test environment for @se/ui: Testing Library matchers, cleanup, and the browser APIs that jsdom
// lacks but Radix and sonner call (layout observers, pointer capture, media queries).
import '@testing-library/jest-dom/vitest';
import { cleanup } from '@testing-library/react';
import { afterEach } from 'vitest';

// Vitest runs without globals, so Testing Library cannot register its own cleanup.
afterEach(() => {
  cleanup();
});

/** Adds `value` as `target[name]` unless the environment already provides it. */
function provideIfMissing(target: object, name: string, value: unknown): void {
  if (!(name in target)) {
    Object.defineProperty(target, name, { value, configurable: true, writable: true });
  }
}

/** No-op ResizeObserver: jsdom has no layout, so there is never anything to report. */
class ResizeObserverStub implements ResizeObserver {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}

provideIfMissing(globalThis, 'ResizeObserver', ResizeObserverStub);
// Radix Select and Slider use pointer capture; jsdom does not implement it.
provideIfMissing(Element.prototype, 'hasPointerCapture', () => false);
provideIfMissing(Element.prototype, 'setPointerCapture', () => undefined);
provideIfMissing(Element.prototype, 'releasePointerCapture', () => undefined);
// Radix Select scrolls the selected option into view when it opens.
provideIfMissing(Element.prototype, 'scrollIntoView', () => undefined);
// sonner reads the colour scheme; jsdom has no `matchMedia`.
provideIfMissing(window, 'matchMedia', (query: string) => ({
  matches: false,
  media: query,
  onchange: null,
  addEventListener: () => undefined,
  removeEventListener: () => undefined,
  dispatchEvent: () => false,
}));

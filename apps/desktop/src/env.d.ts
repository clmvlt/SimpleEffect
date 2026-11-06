import type { SimpleEffectBridge } from '../electron/shared/bridge-api';

declare global {
  interface Window {
    /** Bridge exposed by the preload script (`electron/preload/index.ts`). */
    readonly simpleEffect: SimpleEffectBridge;
  }

  /** The app version from `apps/desktop/package.json`, injected at build time (Vite `define`). */
  const __APP_VERSION__: string;
}

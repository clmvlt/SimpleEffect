import type { SimpleEffectBridge } from '../electron/shared/bridge-api';

declare global {
  interface Window {
    /** Bridge exposed by the preload script (`electron/preload/index.ts`). */
    readonly simpleEffect: SimpleEffectBridge;
  }
}

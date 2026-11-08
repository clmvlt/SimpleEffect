import type { SmartCutSettings } from '@se/protocol';
import { createStore } from 'zustand/vanilla';

/** The smart-cut settings as last read from (or saved to) the core; `null` until read. */
export interface SmartCutSettingsState {
  readonly settings: SmartCutSettings | null;
}

/** The smart-cut settings store (app settings, shared by the silence and filler panels). */
export const smartCutSettingsStore = createStore<SmartCutSettingsState>()(() => ({
  settings: null,
}));

/** Records the settings the core answered with. */
export function setSmartCutSettings(settings: SmartCutSettings): void {
  smartCutSettingsStore.setState({ settings });
}

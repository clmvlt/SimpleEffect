import { createNamespacedStorage, type KeyValueStorage } from './namespaced-storage';

/**
 * UI preferences of this machine (panel sizes…), in the renderer's `localStorage` under `se.ui.`.
 * They live with Electron's user data, not in projects: a layout follows the user, not the file.
 */
export function uiPreferencesStorage(): KeyValueStorage {
  return createNamespacedStorage(window.localStorage, 'se.ui');
}

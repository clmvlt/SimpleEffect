/** The Web Storage subset used for small UI preferences (panel sizes…). */
export type KeyValueStorage = Pick<Storage, 'getItem' | 'setItem'>;

/**
 * Wraps a Web Storage (`localStorage` in the app) under a key prefix, so UI preferences of
 * SimpleEffect never collide with other keys. Storage errors (quota exceeded, storage disabled)
 * are swallowed: losing a panel size must never break the interface.
 */
export function createNamespacedStorage(
  storage: KeyValueStorage,
  namespace: string,
): KeyValueStorage {
  const prefixed = (key: string) => `${namespace}.${key}`;
  return {
    getItem: (key) => {
      try {
        return storage.getItem(prefixed(key));
      } catch {
        return null;
      }
    },
    setItem: (key, value) => {
      try {
        storage.setItem(prefixed(key), value);
      } catch {
        // Preferences are best effort: the default layout is used next time.
      }
    },
  };
}

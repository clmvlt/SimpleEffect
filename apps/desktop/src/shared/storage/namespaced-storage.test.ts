import { describe, expect, it } from 'vitest';

import { createNamespacedStorage, type KeyValueStorage } from './namespaced-storage';

function memoryStorage(): KeyValueStorage & { entries: Map<string, string> } {
  const entries = new Map<string, string>();
  return {
    entries,
    getItem: (key) => entries.get(key) ?? null,
    setItem: (key, value) => {
      entries.set(key, value);
    },
  };
}

describe('createNamespacedStorage', () => {
  it('prefixes every key with the namespace', () => {
    const backing = memoryStorage();
    const storage = createNamespacedStorage(backing, 'se.ui');
    storage.setItem('dock-layout.v1.editor.rows', '{}');
    expect([...backing.entries.keys()]).toEqual(['se.ui.dock-layout.v1.editor.rows']);
    expect(storage.getItem('dock-layout.v1.editor.rows')).toBe('{}');
    expect(storage.getItem('absent')).toBeNull();
  });

  it('survives a storage that throws', () => {
    const failing: KeyValueStorage = {
      getItem: () => {
        throw new Error('storage disabled');
      },
      setItem: () => {
        throw new Error('quota exceeded');
      },
    };
    const storage = createNamespacedStorage(failing, 'se.ui');
    expect(() => {
      storage.setItem('key', 'value');
    }).not.toThrow();
    expect(storage.getItem('key')).toBeNull();
  });
});

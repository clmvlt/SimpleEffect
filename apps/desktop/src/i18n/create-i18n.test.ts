import { describe, expect, it } from 'vitest';

import { createI18n } from './create-i18n';
import fr from './locales/fr.json';

/** Every leaf of a nested translation object, as dotted keys. */
function leafKeys(node: object, prefix = ''): string[] {
  return Object.entries(node).flatMap(([key, value]: [string, unknown]) =>
    typeof value === 'object' && value !== null
      ? leafKeys(value, `${prefix}${key}.`)
      : [`${prefix}${key}`],
  );
}

describe('createI18n', () => {
  it('is ready synchronously in French', () => {
    const i18n = createI18n();
    expect(i18n.isInitialized).toBe(true);
    expect(i18n.language).toBe('fr');
    expect(i18n.t('home.newProject')).toBe('Nouveau projet');
    expect(i18n.t('statusBar.version', { version: '1.2.3' })).toBe('Version 1.2.3');
  });

  it('has a non-empty string for every key', () => {
    const i18n = createI18n();
    for (const key of leafKeys(fr)) {
      const value: unknown = i18n.getResource('fr', 'translation', key);
      expect(typeof value === 'string' && value.trim() !== '', key).toBe(true);
    }
  });
});

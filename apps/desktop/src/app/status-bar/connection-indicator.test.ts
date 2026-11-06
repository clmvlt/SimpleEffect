import type { ConnectionState } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { createI18n } from '../../i18n';
import { connectionIndicator } from './connection-indicator';

describe('connectionIndicator', () => {
  it.each<[ConnectionState, string, string, boolean]>([
    ['connecting', 'warning', 'Connexion au cœur…', true],
    ['open', 'success', 'Cœur connecté', false],
    ['reconnecting', 'warning', 'Reconnexion au cœur…', true],
    ['closed', 'danger', 'Connexion au cœur fermée', false],
  ])('shows %s as a %s dot labelled « %s »', (state, tone, label, pulse) => {
    const indicator = connectionIndicator(state);
    expect(indicator.tone).toBe(tone);
    expect(indicator.pulse).toBe(pulse);
    expect(createI18n().t(indicator.labelKey)).toBe(label);
  });
});

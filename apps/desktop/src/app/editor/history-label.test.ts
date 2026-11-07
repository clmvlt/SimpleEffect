import { describe, expect, it } from 'vitest';

import { createI18n } from '../../i18n';
import { historyLabel } from './history-label';

describe('historyLabel', () => {
  const { t } = createI18n();

  it('translates the labels of transactions built by the core', () => {
    expect(historyLabel(t, 'Create sequence')).toBe('Création de séquence');
    expect(historyLabel(t, 'Rename project')).toBe('Renommage du projet');
  });

  it('shows other labels as they are', () => {
    expect(historyLabel(t, 'Couper les silences')).toBe('Couper les silences');
  });
});

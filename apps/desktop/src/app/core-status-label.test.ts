import { describe, expect, it } from 'vitest';

import { describeCoreStatus } from './core-status-label';

describe('describeCoreStatus', () => {
  it('shows the core version once connected', () => {
    expect(describeCoreStatus({ kind: 'connected', version: '0.1.0' })).toBe(
      'Cœur connecté — se-core 0.1.0',
    );
  });

  it('distinguishes the first connection from a reconnection', () => {
    expect(describeCoreStatus({ kind: 'connecting' })).toBe('Connexion au cœur…');
    expect(describeCoreStatus({ kind: 'reconnecting' })).toBe('Reconnexion…');
  });
});

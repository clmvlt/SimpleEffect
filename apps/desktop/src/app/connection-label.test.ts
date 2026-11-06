import type { CoreInfo } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { describeConnection, describeCoreInfo } from './connection-label';

const INFO: CoreInfo = {
  version: '0.1.0',
  os: 'windows',
  arch: 'x86_64',
  appDataDir: 'C:\\Users\\me\\AppData\\Roaming\\SimpleEffect',
  logsDir: 'C:\\Users\\me\\AppData\\Roaming\\SimpleEffect\\logs',
};

describe('describeConnection', () => {
  it('shows the core version once the info is known', () => {
    expect(describeConnection('open', INFO)).toBe('Cœur connecté — se-core 0.1.0');
    expect(describeConnection('open', null)).toBe('Cœur connecté');
  });

  it('distinguishes the first connection, a reconnection and a closed client', () => {
    expect(describeConnection('connecting', null)).toBe('Connexion au cœur…');
    expect(describeConnection('reconnecting', INFO)).toBe('Reconnexion…');
    expect(describeConnection('closed', null)).toBe('Connexion au cœur fermée');
  });
});

describe('describeCoreInfo', () => {
  it('lists the platform and the data and log folders', () => {
    expect(describeCoreInfo(INFO)).toEqual([
      { label: 'Plateforme', value: 'windows (x86_64)' },
      { label: 'Données', value: INFO.appDataDir },
      { label: 'Journaux', value: INFO.logsDir },
    ]);
  });
});

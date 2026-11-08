import { describe, expect, it } from 'vitest';

import { APP_ID, notificationAppId } from './app-identity';

describe('notificationAppId', () => {
  it('is the app id when installed on Windows, the executable in development, none elsewhere', () => {
    expect(notificationAppId('win32', true, 'C:/app/SimpleEffect.exe')).toBe(APP_ID);
    expect(notificationAppId('win32', false, 'C:/dev/electron.exe')).toBe('C:/dev/electron.exe');
    expect(notificationAppId('darwin', true, '/Applications/SimpleEffect.app')).toBeNull();
  });
});

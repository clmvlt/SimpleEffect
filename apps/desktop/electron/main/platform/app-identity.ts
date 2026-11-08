/** The app id of electron-builder (`electron-builder.yml`), also Windows' AppUserModelID. */
export const APP_ID = 'com.simpleeffect.app';

/**
 * The AppUserModelID to set before showing notifications: Windows attributes toasts to it (and
 * shows « electron.app.Electron » without it in development); other systems need none.
 */
export function notificationAppId(platform: NodeJS.Platform): string | null {
  return platform === 'win32' ? APP_ID : null;
}

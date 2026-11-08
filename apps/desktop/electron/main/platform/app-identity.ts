/** The app id of electron-builder (`electron-builder.yml`), also Windows' AppUserModelID. */
export const APP_ID = 'com.simpleeffect.app';

/**
 * The AppUserModelID to set before showing notifications: Windows attributes toasts to it. The
 * installed app has a Start menu shortcut carrying {@link APP_ID}; a development run has none,
 * so it uses the executable's path as Electron recommends. Other systems need none.
 */
export function notificationAppId(
  platform: NodeJS.Platform,
  packaged: boolean,
  executablePath: string,
): string | null {
  if (platform !== 'win32') return null;
  return packaged ? APP_ID : executablePath;
}

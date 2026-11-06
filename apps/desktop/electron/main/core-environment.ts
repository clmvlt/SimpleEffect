/** Environment variable telling se-core where the app data folder is (logs, settings, library). */
export const CORE_DATA_DIR_ENV = 'SE_DATA_DIR';

/**
 * Environment of the se-core process: the parent's environment plus the app data folder, so the
 * app and the core share one folder (`app.getPath('userData')`).
 */
export function coreEnvironment(
  dataDir: string,
  baseEnvironment: NodeJS.ProcessEnv,
): NodeJS.ProcessEnv {
  return { ...baseEnvironment, [CORE_DATA_DIR_ENV]: dataDir };
}

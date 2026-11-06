import { join } from 'node:path';

import { executableName } from './platform/executable-name';

/** Development-only environment variable pointing at a custom se-core binary. */
export const CORE_PATH_ENV = 'SE_CORE_PATH';

/** Where the app runs from; gathered from Electron's `app` and `process` at startup. */
export interface CoreBinaryContext {
  readonly isPackaged: boolean;
  /** `process.resourcesPath`: the packaged app's resources folder. */
  readonly resourcesPath: string;
  /** `app.getAppPath()`: `apps/desktop` during development. */
  readonly appPath: string;
  readonly platform: NodeJS.Platform;
  /** Value of {@link CORE_PATH_ENV}, honoured only during development. */
  readonly override: string | undefined;
}

/**
 * Locates the se-core binary.
 *
 * - Packaged: `<resources>/bin/se-core[.exe]`, copied there by electron-builder `extraResources`.
 * - Development: `<repo>/target/debug/se-core[.exe]`, built by `pnpm dev` before Electron starts.
 */
export function resolveCoreBinaryPath(context: CoreBinaryContext): string {
  const fileName = executableName('se-core', context.platform);
  if (context.isPackaged) {
    return join(context.resourcesPath, 'bin', fileName);
  }
  if (context.override !== undefined && context.override.length > 0) {
    return context.override;
  }
  return join(context.appPath, '..', '..', 'target', 'debug', fileName);
}

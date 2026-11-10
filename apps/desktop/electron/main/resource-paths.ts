import { join } from 'node:path';

/** Where the app runs from (a subset of `CoreBinaryContext`). */
export interface ResourceContext {
  readonly isPackaged: boolean;
  /** `process.resourcesPath`: the packaged app's resources folder. */
  readonly resourcesPath: string;
  /** `app.getAppPath()`: `apps/desktop` during development. */
  readonly appPath: string;
}

/** The licence files of the app (prompt 30, `electron-builder.yml` `extraResources`). */
export interface LicencePaths {
  /** The folder of licence texts shown by « Ouvrir le dossier des licences ». */
  readonly folder: string;
  /** The generated `third-party-notices.json`. */
  readonly notices: string;
}

/**
 * Locates the licence files.
 *
 * - Packaged: `<resources>/LICENSES/`, holding the repository's `LICENSES/` folder and the
 *   generated notices.
 * - Development: the repository's `LICENSES/` and `apps/desktop/out/notices/` (written by
 *   `pnpm build`; absent until then).
 */
export function resolveLicencePaths(context: ResourceContext): LicencePaths {
  if (context.isPackaged) {
    const folder = join(context.resourcesPath, 'LICENSES');
    return { folder, notices: join(folder, 'third-party-notices.json') };
  }
  return {
    folder: join(context.appPath, '..', '..', 'LICENSES'),
    notices: join(context.appPath, 'out', 'notices', 'third-party-notices.json'),
  };
}

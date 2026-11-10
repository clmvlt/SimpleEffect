/**
 * Which npm packages a bundle contains (prompt 30): the licence notices list exactly what ships.
 * Pure helpers of `bundled-packages-plugin.ts`.
 */

/** A package found in a bundle. */
export interface BundledPackage {
  /** Its name (`react`, `@pixi/core`). */
  readonly name: string;
  /** Its folder, absolute, `/`-separated. */
  readonly directory: string;
}

/**
 * The package a module id belongs to, or `null` for the app's own code (workspace packages are
 * not under `node_modules`) and virtual modules. pnpm nests packages
 * (`node_modules/.pnpm/react@19/node_modules/react/…`): the last `node_modules` segment wins.
 */
export function packageOfModule(moduleId: string): BundledPackage | null {
  if (moduleId.startsWith('\0')) return null;
  const path = moduleId.split('?')[0]?.replaceAll('\\', '/') ?? '';
  const marker = '/node_modules/';
  const index = path.lastIndexOf(marker);
  if (index === -1) return null;
  const rest = path.slice(index + marker.length).split('/');
  const [first, second] = rest;
  if (first === undefined || first === '' || first.startsWith('.')) return null;
  const name = first.startsWith('@') && second !== undefined ? `${first}/${second}` : first;
  return { name, directory: `${path.slice(0, index + marker.length)}${name}` };
}

/** The distinct packages of `moduleIds`, sorted by name then folder. */
export function packagesOfModules(moduleIds: Iterable<string>): BundledPackage[] {
  const byDirectory = new Map<string, BundledPackage>();
  for (const id of moduleIds) {
    const found = packageOfModule(id);
    if (found !== null) byDirectory.set(found.directory, found);
  }
  return [...byDirectory.values()].sort(
    (left, right) =>
      left.name.localeCompare(right.name) || left.directory.localeCompare(right.directory),
  );
}

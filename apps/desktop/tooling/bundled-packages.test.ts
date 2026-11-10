import { describe, expect, it } from 'vitest';

import { packageOfModule, packagesOfModules } from './bundled-packages';

describe('bundled packages', () => {
  it('finds the package of a module installed by pnpm', () => {
    expect(
      packageOfModule('C:\\repo\\node_modules\\.pnpm\\react@19.3.1\\node_modules\\react\\index.js'),
    ).toEqual({
      name: 'react',
      directory: 'C:/repo/node_modules/.pnpm/react@19.3.1/node_modules/react',
    });
    expect(
      packageOfModule('/r/node_modules/.pnpm/@pixi+core@8/node_modules/@pixi/core/lib/a.mjs?x=1'),
    ).toEqual({
      name: '@pixi/core',
      directory: '/r/node_modules/.pnpm/@pixi+core@8/node_modules/@pixi/core',
    });
  });

  it('ignores the app’s own code and virtual modules', () => {
    expect(packageOfModule('/repo/packages/engine/src/index.ts')).toBeNull();
    expect(packageOfModule('\0vite/preload-helper')).toBeNull();
    expect(packageOfModule('/repo/node_modules/.vite/deps/chunk.js')).toBeNull();
  });

  it('lists each package once, sorted', () => {
    const packages = packagesOfModules([
      '/n/node_modules/zustand/index.js',
      '/n/node_modules/react/a.js',
      '/n/node_modules/react/b.js',
      '/repo/src/main.tsx',
    ]);
    expect(packages.map((found) => found.name)).toEqual(['react', 'zustand']);
  });
});

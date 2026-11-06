// @ts-check
import js from '@eslint/js';
import prettier from 'eslint-config-prettier';
import boundaries from 'eslint-plugin-boundaries';
import reactHooks from 'eslint-plugin-react-hooks';
import { defineConfig, globalIgnores } from 'eslint/config';
import globals from 'globals';
import tseslint from 'typescript-eslint';

const RENDERER = 'apps/desktop/src';

/**
 * Architecture rules of the renderer (docs/architecture.md §4.2–4.3):
 * - a feature imports another feature only through its `index.ts`;
 * - `shared/` never imports features or `app/`; features never import `app/`.
 */
const rendererBoundaries = {
  files: [`${RENDERER}/**/*.{ts,tsx}`],
  plugins: { boundaries },
  settings: {
    'boundaries/include': [`${RENDERER}/**/*`],
    'boundaries/elements': [
      { type: 'app', pattern: `${RENDERER}/app` },
      { type: 'feature', pattern: `${RENDERER}/features/*`, capture: ['featureName'] },
      { type: 'shared', pattern: `${RENDERER}/shared` },
      { type: 'i18n', pattern: `${RENDERER}/i18n` },
    ],
    'import/resolver': { typescript: { alwaysTryTypes: true } },
  },
  rules: {
    'boundaries/dependencies': [
      'error',
      {
        default: 'allow',
        policies: [
          {
            from: { element: { types: ['feature', 'shared', 'i18n'] } },
            disallow: { to: { element: { type: 'app' } } },
            message: 'Only app/ composes the application: do not import app/ from here.',
          },
          {
            from: { element: { types: ['shared', 'i18n'] } },
            disallow: { to: { element: { type: 'feature' } } },
            message: 'shared/ and i18n/ must not depend on features.',
          },
          {
            from: { element: { type: 'feature' } },
            disallow: {
              to: {
                element: {
                  type: 'feature',
                  captured: { featureName: '!{{ from.element.captured.featureName }}' },
                  fileInternalPath: '!index.ts',
                },
              },
            },
            message: 'Import another feature only through its index.ts (its public API).',
          },
        ],
      },
    ],
  },
};

export default defineConfig(
  globalIgnores([
    '**/node_modules/',
    '**/out/',
    '**/dist/',
    '**/release/',
    '**/.tsbuild/',
    'target/',
    'packages/protocol/src/generated/',
  ]),
  js.configs.recommended,
  tseslint.configs.strictTypeChecked,
  {
    languageOptions: {
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
  },
  {
    files: ['**/*.{js,mjs,cjs}'],
    extends: [tseslint.configs.disableTypeChecked],
    languageOptions: { globals: globals.node },
  },
  {
    files: ['apps/desktop/electron/**/*.ts', 'apps/desktop/tooling/**/*.ts'],
    languageOptions: { globals: globals.node },
  },
  {
    files: [`${RENDERER}/**/*.{ts,tsx}`],
    extends: [reactHooks.configs.flat['recommended-latest']],
    languageOptions: { globals: globals.browser },
  },
  rendererBoundaries,
  prettier,
);

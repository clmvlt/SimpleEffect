// @ts-check
import js from '@eslint/js';
import prettier from 'eslint-config-prettier';
import boundaries from 'eslint-plugin-boundaries';
import i18next from 'eslint-plugin-i18next';
import reactHooks from 'eslint-plugin-react-hooks';
import { defineConfig, globalIgnores } from 'eslint/config';
import globals from 'globals';
import tseslint from 'typescript-eslint';

const RENDERER = 'apps/desktop/src';
const UI = 'packages/ui';

/** Libraries wrapped by @se/ui; nothing else may import them (golden rule 9). */
const PRIMITIVE_LIBRARIES = [
  'radix-ui',
  '@radix-ui/*',
  'lucide-react',
  'sonner',
  'class-variance-authority',
  'react-resizable-panels',
];

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

/**
 * No hard-coded UI text in the renderer (docs/architecture.md §4.2): JSX text and the attributes
 * that users see or hear must come from `t()`. Exceptions: tests (they assert on rendered text),
 * strings made only of digits/punctuation or written in CAPITALS (the plugin's defaults), and every
 * attribute not listed below (class names, ids, routes, variants…).
 */
const rendererLiteralStrings = {
  files: [`${RENDERER}/**/*.tsx`],
  ignores: [`${RENDERER}/**/*.test.tsx`],
  plugins: { i18next },
  rules: {
    'i18next/no-literal-string': [
      'error',
      {
        mode: 'jsx-only',
        'jsx-attributes': {
          include: [
            'alt',
            'aria-label',
            'aria-description',
            'aria-placeholder',
            'aria-roledescription',
            'aria-valuetext',
            'content',
            'description',
            'label',
            'placeholder',
            'title',
            '.+Label',
          ],
          exclude: ['.*'],
        },
        message:
          'User-visible text must come from i18n: add a key to src/i18n/locales/fr.json and use t().',
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
    '**/storybook-static/',
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
    files: [`${RENDERER}/**/*.{ts,tsx}`, `${UI}/**/*.{ts,tsx}`],
    extends: [reactHooks.configs.flat['recommended-latest']],
    languageOptions: { globals: globals.browser },
  },
  {
    // Golden rule 9: the app gets primitives, icons and toasts from @se/ui only.
    files: [`${RENDERER}/**/*.{ts,tsx}`],
    rules: {
      'no-restricted-imports': [
        'error',
        {
          patterns: [
            {
              group: PRIMITIVE_LIBRARIES,
              message:
                'Import UI from @se/ui (icons from @se/ui/icons): primitive libraries stay inside packages/ui.',
            },
          ],
        },
      ],
    },
  },
  rendererBoundaries,
  rendererLiteralStrings,
  prettier,
);

# @se/ui

## Purpose
The SimpleEffect design system (`docs/adr/0009-design-system.md`): design tokens, primitives built on Radix, and generic editor widgets. Presentational only: it knows nothing about the document model or the protocol.

Currently an empty skeleton, implemented by prompts 03 (foundations) and 04 (editor components, shell, i18n).

## Public API
None yet (`src/index.ts` exports nothing). Planned: tokens, primitives, editor widgets, all exported from `src/index.ts`.

## Rules and invariants
- The only place that may import Radix or other primitive libraries (golden rule 9).
- Use design tokens, never raw colour values.
- Each component lives in `src/<layer>/<component>/` with CVA variants, a story and a test.
- No domain or protocol imports; no user-visible text without a prop (app strings come from i18n).

## Allowed dependencies
No workspace package. See `docs/architecture.md` §4.1.

## How to test
- `pnpm vitest run --project ui` (or `pnpm test` at the root); `pnpm storybook` from prompt 03.
- Tests live next to the components as `*.test.tsx`.

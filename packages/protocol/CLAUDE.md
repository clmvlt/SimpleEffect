# @se/protocol

## Purpose
TypeScript side of the se-core protocol: the types generated from `se-domain` and `se-protocol` (ts-rs) and the typed `CoreClient` used by the renderer. It contains no React and no rendering code.

Currently an empty skeleton: prompt 02 adds the generated types and `CoreClient`. Until then the renderer's ping lives in `apps/desktop/src/shared/core-connection/` (marked `TODO(prompt-02)`).

## Public API
None yet (`src/index.ts` exports nothing). Planned: generated types re-exported from `src/generated/`, `CoreClient` (`send`, `subscribe`, reconnect with snapshot resync).

## Rules and invariants
- `src/generated/` is written by `pnpm codegen` only: never hand-edit it; change the Rust types instead.
- No imports from React, `@se/engine` or `@se/ui`.
- The package is consumed as TypeScript source (`exports` → `src/index.ts`); no build step.

## Allowed dependencies
No workspace package. See `docs/architecture.md` §4.1.

## How to test
- `pnpm vitest run --project protocol` (or `pnpm test` at the root).
- Tests live next to the code as `*.test.ts`.

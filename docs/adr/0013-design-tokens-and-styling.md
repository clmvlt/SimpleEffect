# 0013 — Design tokens and styling conventions of `@se/ui`

- Status: Accepted
- Date: 2026-11-06

## Context
ADR-0009 chose a shadcn-style design system on Tailwind CSS v4 and Radix. Prompt 03 implemented its foundations and had to settle how tokens are stored and enforced, how a future light theme and accent choices fit in, how icons reach the app, and how the renderer's Content-Security-Policy copes with the libraries' runtime styles. These choices are read by every agent writing UI code, so they must be explicit and machine-checked.

## Decision
- **Tokens are CSS, not TypeScript.** `packages/ui/src/tokens/*.css` declares every token as a Tailwind v4 `@theme static` variable in OKLCH (`--color-panel`, `--text-sm`, `--spacing-control-md`, `--radius-md`, `--ease-standard`, `--animate-content-in`…) plus plain `:root` variables where Tailwind has no namespace (`--z-*`, `--duration-*`). The stylesheet is the single source of truth: tests and the Storybook Tokens page parse it (`token-css.ts`) instead of keeping a copy.
- **Only tokens can be used.** Each Tailwind namespace we define is reset first (`--color-*: initial`…), so default utilities such as `bg-neutral-900` or `text-base` do not exist. `scripts/check-raw-colors.mjs` (part of `pnpm check`) rejects hex colours and colour functions outside `tokens/` folders in `packages/ui/src` and `apps/desktop/src`.
- **Themes and accents override variables.** Components reference `var(--color-…)` through utilities; a theme or accent is a selector overriding variables (`[data-accent='gold']`, later `[data-theme='light']`). Content drawn on the accent always uses `accent-foreground`, so a light accent works as well as a dark one.
- **Accent: bright yellow.** Indigo-violet, violet and blue were presented; the user chose yellow. It is the default in `colors.css` (dark `accent-foreground`), with `lemon` and `gold` shades in `accents.css`. Warning moved to orange and caption items to violet so that no other colour reads as the accent.
- **Contrast and gamut are invariants.** `tokens/contrast.test.ts` checks that every colour token is inside the sRGB gamut, converts OKLCH to sRGB and asserts WCAG AA (4.5:1) for text tokens on every surface, for labels on filled controls, and 3:1 for the focus ring, in every accent. Every story is also checked by axe in `pnpm test` (`src/stories.a11y.test.tsx`; colour contrast excluded there because jsdom has no layout).
- **Stacking order** is `--z-panel` < `--z-modal` < `--z-dropdown` < `--z-toast` < `--z-tooltip`, so menus and selects opened from a dialog float above it.
- **Entry points.** `@se/ui` (components), `@se/ui/icons` (lucide re-export, kept out of the main namespace where names would clash), `@se/ui/styles.css` (Tailwind + fonts + tokens + base; apps import only this file). ESLint forbids importing `radix-ui`, `@radix-ui/*`, `lucide-react`, `sonner` and `class-variance-authority` from the app.
- **CSP allows inline styles.** Radix (scroll areas, select viewports) and sonner insert `<style>` elements at runtime, which `style-src 'self'` blocks. The renderer CSP now has `style-src 'self' 'unsafe-inline'` in production too (VS Code makes the same choice). Scripts remain `'self'` only, connections remain limited to `127.0.0.1`, and fonts and images come from the bundle, so an injected style has nothing to load or leak to.

## Consequences
- A new colour, size or animation is one line in a token file; tailwind-merge must also learn custom names whose utility it cannot guess (`lib/cn.ts`, tested).
- Agents cannot drift to ad-hoc colours: lint, the guard and the contrast test fail first.
- A light theme is an additional override block plus the same contrast test run against it.
- The CSP is slightly weaker for styles; revisit if the renderer ever loads untrusted HTML.

## Alternatives considered
- **Tokens as TypeScript objects generating CSS**: one more build step and a second representation to keep in sync; Tailwind v4 already reads CSS variables natively. Rejected.
- **Keeping Tailwind's default palette next to the tokens**: convenient, but agents would mix `zinc-800` with tokens. Rejected.
- **Nonces or hashes for the runtime `<style>` elements**: a file-based renderer has no per-response nonce, and hashes of third-party internals break silently on upgrades. Rejected.
- **Importing sonner's stylesheet and keeping `style-src 'self'`**: sonner still tries to inject its style (console CSP errors on every start) and Radix scroll areas would keep their native scrollbars. Rejected.

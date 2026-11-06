# @se/ui

## Purpose
The SimpleEffect design system (`docs/adr/0009-design-system.md`, `docs/adr/0013-design-tokens-and-styling.md`): design tokens, accessible primitives built on Radix, and (from prompt 04) generic editor widgets. Presentational only: it knows nothing about the document model or the protocol.

Visual direction: **modern, clean, dense** — dark neutral surfaces with a slight cool tint, one vivid accent (yellow), soft contrasts, thin borders, almost no shadows, 2–8 px corners, 24/28/32 px controls, 13 px base text.

## Public API
Three entry points (`package.json` `exports`):
- `@se/ui` (`src/index.ts`): every primitive, `cn()`, `focusRing`, `Slot`/`Slottable`.
- `@se/ui/icons` (`src/icons.ts`): lucide icons (`import { Scissors } from '@se/ui/icons'`), drawn with `<Icon icon={Scissors} size="sm" />`.
- `@se/ui/styles.css` (`src/styles.css`): Tailwind v4 + bundled fonts (Inter Variable, JetBrains Mono) + tokens + base styles. Apps import only this file and never `tailwindcss` itself.

Primitives (`src/primitives/`, shadcn-style compound names): Button, IconButton, Icon, Input, Textarea, Label, Field, Select, Checkbox, Switch, Slider, RadioGroup, Tooltip (+ `TooltipProvider`, keyboard-shortcut slot), Popover, Dialog, AlertDialog, DropdownMenu, ContextMenu, Tabs, ToggleGroup, Separator, ScrollArea, Kbd, Badge, Toast (`Toaster` + `toast()`, wraps sonner), Progress, Spinner, Skeleton. Browse them with `pnpm storybook`.

## Accent
**Chosen accent: bright yellow (`yellow`, OKLCH hue 96)**, picked by the user in prompt 03 over the indigo-violet, violet and blue candidates. It is the default of the five accent tokens in `src/tokens/colors.css`. Because it is light, `accent-foreground` is dark: always draw content on `bg-accent` with `text-accent-foreground` (never `text-text`). Nearby shades `lemon` (108) and `gold` (84) live in `src/tokens/accents.css` (`[data-accent='…']`) for the "Foundations/Theme" comparison and a future accent preference. To keep the accent unambiguous, `warning` is orange (65) and `item-caption` violet (290).

## Layout
| Path | Role |
|---|---|
| `src/tokens/*.css` | All design tokens (`@theme static`, OKLCH): `colors.css`, `accents.css`, `typography.css`, `layout.css`, `motion.css`. The **only** place where colour literals are allowed |
| `src/tokens/color-math.ts`, `token-css.ts` | OKLCH → sRGB, WCAG contrast; parsing of the token stylesheets (used by tests and the Tokens page) |
| `src/tokens/contrast.test.ts` | Palette invariants: every colour inside the sRGB gamut; AA text on every surface, labels on filled controls, focus ring, for every accent shade; default accent = `yellow` |
| `src/lib/` | `cn()` (clsx + tailwind-merge aware of our custom tokens), `focusRing`, `floatingSurfaceClasses` (popovers, menus, select lists), `Slot` |
| `src/primitives/<name>/` | One component per folder: `<name>.tsx`, `<name>.stories.tsx`, `<name>.test.tsx`, `index.ts` |
| `src/docs/` | Storybook foundation pages: `principles.mdx`, `tokens.stories.tsx` (generated from the token files), `theme.stories.tsx` (accent variants); `catalog/` holds their docs-only components |
| `src/stories.a11y.test.tsx` | Renders every story and runs axe (WCAG 2.2 A/AA, colour contrast excluded: no layout in jsdom) |
| `.storybook/` | Storybook 10 (Vite builder, dark theme, a11y + docs addons, telemetry off) |
| `vitest.setup.ts` | jest-dom matchers, cleanup, jsdom stubs (ResizeObserver, pointer capture, `matchMedia`) |

## Tokens cheat sheet
- Surfaces: `bg` (window) < `panel` < `panel-raised` < `overlay` (menus, dialogs); `input` (text fields), `control`/`control-hover` (secondary buttons), `border`, `border-strong`, `hover` (translucent row/ghost hover), `scrim` (modal backdrop).
- Text: `text`, `text-muted`, `text-subtle` (still AA), `text-inverted`.
- Accent: `accent`, `accent-hover`, `accent-muted` (selection), `accent-foreground`, `focus-ring`.
- States: `success|warning|danger|info` (readable as text) + `-muted` fills; `danger-solid`, `danger-solid-hover`, `danger-foreground` for destructive buttons.
- Timeline: `item-video|item-audio|item-graphic|item-caption|ai`, each with `-muted` (fill) and `-strong` (border, text).
- Sizes: `h-control-xs|sm|md` (24/28/32 px, also `size-`, `w-`…), 4 px grid (`p-1` = 4 px), radii `rounded-xs|sm|md|lg`, `shadow-overlay` only on floating layers.
- Type: `text-2xs|xs|sm|md|lg|xl` = 11/12/13/14/16/20 px; `font-normal|medium|semibold`; `timecode` utility (mono + tabular numerals) for timecodes and numeric readouts.
- Motion: `duration-(--duration-fast|normal|slow)`, `ease-standard|out|in`, `animate-overlay-in|out`, `animate-content-in|out`, `animate-spin|pulse`; all disabled under `prefers-reduced-motion`.
- Layers: `z-(--z-panel)` < `z-(--z-modal)` < `z-(--z-dropdown)` < `z-(--z-toast)` < `z-(--z-tooltip)`.

## Rules and invariants
- The only package that may import Radix, lucide, sonner or cva (golden rule 9; ESLint enforces it in the app).
- Tokens only: no colour literal outside `src/tokens/` (`pnpm check` runs `scripts/check-raw-colors.mjs`); the default Tailwind palette and scales are removed (`text-base`, `bg-zinc-800` do not exist).
- No domain or protocol imports; no user-visible text inside components: labels, placeholders and accessible names are props (the app passes i18n strings). Icon-only controls require a `label`.
- Accessibility: keyboard navigation and visible focus (`focusRing`) everywhere, AA text contrast (tested), every story passes axe.
- Floating layers: `bg-overlay border border-border rounded-lg shadow-overlay`, `data-[state=open]:animate-content-in`, and the matching `z-(--z-…)` layer.
- Components are function components with `cva` variants and `cn()`; defaults: size `sm` (28 px). React Compiler is on: no manual memoisation.

## Adding a component
1. Create `src/primitives/<name>/` (or `src/editor/<name>/` from prompt 04) with `<name>.tsx`, `<name>.stories.tsx` (title `Primitives/<Name>`, show variants and states), `<name>.test.tsx` (behaviour: keyboard, focus, open/close…), `index.ts`.
2. Wrap the Radix primitive (`import { X as XPrimitive } from 'radix-ui'`), style it with tokens, expose shadcn-style compound parts and a doc comment on every export.
3. Add `export * from './primitives/<name>';` to `src/index.ts`.
4. Need a new token? Add it to the right `src/tokens/*.css` file (and to `lib/cn.ts` if tailwind-merge cannot classify its utility); colours must keep `contrast.test.ts` green.
5. `pnpm vitest run --project ui` (component tests + axe on the new stories), then `pnpm check`.

## Allowed dependencies
No workspace package. Runtime: `radix-ui`, `class-variance-authority`, `clsx`, `tailwind-merge`, `lucide-react`, `sonner`, `@fontsource-variable/inter`, `@fontsource/jetbrains-mono`; React as a peer. See `docs/architecture.md` §4.1.

## How to test
- `pnpm vitest run --project ui` (or `pnpm test` at the root): component tests, axe on every story, token contrast.
- `pnpm storybook` from the root (http://localhost:6006): check the a11y panel and the "Foundations" pages after visual changes. `pnpm --filter @se/ui build-storybook` builds a static copy into `storybook-static/` (gitignored).

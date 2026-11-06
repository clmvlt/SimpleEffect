# 0014 — Renderer shell, i18n and editor widget conventions

- Status: Accepted
- Date: 2026-11-06

## Context
Prompt 04 replaced the temporary connection screen with the application shell (home, editor layout, status bar) and added the editor-specific widgets of `@se/ui` (ADR-0009). It had to settle how the renderer routes between screens when it is loaded from a file, how UI text is translated and kept out of the code, where panel sizes are remembered, and one interaction rule that every editor widget must share so that undo (ADR-0005) works: a drag must not create dozens of undo steps.

## Decision
- **Routing: React Router in library mode with a hash router.** Routes are plain objects built by `createAppRoutes(deps)` (`apps/desktop/src/app/routes.tsx`): `/` home, `/project/:projectId` editor. Production loads `index.html` from disk, so `main.tsx` uses `createHashRouter` (paths in the URL fragment survive reloads without a server); tests build a memory router from the same routes. Dependencies the routes need (panel-size storage) are injected by the composition root.
- **i18n: i18next + react-i18next, French only, bundled.** `createI18n()` initialises an instance synchronously from `src/i18n/locales/fr.json` (no backend, no lazy loading: golden rule 1). Keys are grouped by area (`home.*`, `editor.*`, `statusBar.*`, `common.*`) and typed through `CustomTypeOptions` (`src/i18n/i18next.d.ts`), so an unknown key fails `tsc`. `eslint-plugin-i18next` (`no-literal-string`, mode `jsx-only`) rejects JSX text and user-visible attributes (`aria-label`, `title`, `placeholder`, `label`, `*Label`…) written as literals in `apps/desktop/src`; tests are exempt, and strings of digits/punctuation or CAPITALS are allowed.
- **`@se/ui` holds no product text.** Widgets take their labels and accessible names as props. Generic accessibility names have an English default (`label = 'Resize panels'`) that the app always overrides with an i18n string; widgets with many names take a `labels` object merged over `DEFAULT_*_LABELS`.
- **Gesture commit convention.** Editor widgets that edit a value continuously (`ScrubInput`, `ColorField`, `EasingCurveEditor`) emit `onChange` for live feedback and `onCommit` **exactly once per gesture**: on pointer release after a drag, after each key press, after a typed value is accepted, on a preset or swatch pick. Callers preview with `onChange` and send the document command on `onCommit`, so one gesture is one undo step.
- **Panel sizes are a machine preference, not project data.** `Dock` (on `react-resizable-panels`) saves only user-made resizes, under a stable `layoutId`, in an injected `DockLayoutStorage` (`dock-layout.v1.<layoutId>`). The app injects `localStorage` under the `se.ui.` namespace (`shared/storage/`), which lives in Electron's user data; a corrupt or missing entry falls back to the default sizes.
- **Widgets stay domain-agnostic.** Timecode widgets receive `format`/`parse`; the easing editor has its own presentational `EasingCurve` type; `ColorField` works on `#rrggbb` strings with OKLCH editing. Mapping to domain types (flicks, `Easing`, colours) is the app's job.

## Consequences
- Adding a screen is one route object; adding text is one key in `fr.json` (typed at once), and the lint makes forgetting it impossible.
- A second language later is one more JSON file plus a language setting; `@se/ui` needs no change.
- Every future property editor must follow the commit convention; reviewers check that the document command is sent from `onCommit`, never from `onChange`.
- Layout ids become a compatibility surface: renaming one resets the users' panel sizes (the `v1` key version allows a deliberate migration).
- jsdom has no layout, so pointer resizing of docks is verified in the running app; tests cover persistence and keyboard handles.

## Alternatives considered
- **Browser router with a custom protocol or local server**: needs a privileged scheme or an HTTP server for history-API paths; the hash router has no such cost for a desktop app. Rejected.
- **React Router framework mode / file-based routing**: brings a build plugin and data-loading conventions we do not need (data comes from se-core). Rejected.
- **FormatJS / Lingui**: message-format power we do not need yet, and Lingui needs a compile step; i18next is the de facto standard with first-class TypeScript key typing. Rejected.
- **Storing panel sizes in the project file**: layouts would differ per project and travel between machines with different screens; they are a user preference. Rejected.
- **Committing on every `onChange`**: floods the undo history and the core with one transaction per pointer move. Rejected.

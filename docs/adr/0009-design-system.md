# 0009 — Design system `@se/ui`

- Status: Accepted
- Date: 2026-11-06

## Context
The user wants a clean UI built on an in-house component library "in the style of shadcn, adapted to video editing". Chosen visual direction: **modern, clean and dense** (Linear/Vercel-like, dark neutral surfaces, one vivid accent, compact controls).

## Decision
- `packages/ui` (`@se/ui`) follows shadcn principles: component source lives in the repo, composition over configuration, variants with `class-variance-authority`, styling with **Tailwind CSS v4** and CSS-variable **design tokens** (OKLCH), accessible primitives from **Radix UI** wrapped by our components.
- The app imports UI **only** from `@se/ui`, never from Radix directly; this keeps primitives replaceable (e.g. Base UI).
- Dark theme first; tokens structured so a light theme can be added later.
- Density: control heights 24/28/32 px, 12–13 px base text, tabular numerals and a monospace face for timecodes.
- Fonts (Inter, JetBrains Mono) and icons (lucide) are bundled; no CDN.
- Semantic colour tokens for item kinds: video, audio, text/graphics, captions, AI.
- Layers: `tokens/` → `primitives/` → `editor/` (Panel, Dock, PropertyRow, ScrubInput, TimecodeInput, KeyframeButton, EasingCurveEditor, TransportBar, Meter, Ruler, Playhead, TrackHeader, ClipShell, AspectRatioPicker, AiTaskCard, PromptComposer…).
- Every component has Storybook stories (with the a11y addon) and tests; stories are the living documentation for humans and AI.

## Consequences
- Full control over look and behaviour; consistent editor-grade widgets.
- We own maintenance of the components (no upstream updates for free).

## Alternatives considered
- **MUI / Mantine / Chakra**: fast to start but opinionated styling, heavier, generic rather than editor-grade. Rejected.
- **Using shadcn/ui directly in the app**: good base for primitives (and we borrow its patterns), but editor widgets would still be custom; a dedicated package keeps boundaries clear.

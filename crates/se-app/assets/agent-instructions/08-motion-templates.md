# Motion templates

When texts, shapes, keyframes and presets are not enough (a lower third with an animated bar, a
title whose words rise one by one, a progress bar, a call-to-action button), **write a motion
template**: a small TypeScript module the editor runs in a sandbox at every frame. The user then
edits its texts, colours, numbers and options in the inspector without touching code — so put
everything they may want to change in `params`.

## Workflow

1. `create_motion_template {"source": "…"}` with the whole module. The editor compiles it, runs it,
   checks the time budget and determinism, and answers the `template_id` with three pictures
   (entrance, middle, exit, default values) — or `TEMPLATE_INVALID` listing every problem with its
   line: fix them all and send the whole module again.
2. Look at the pictures. Then `instantiate_template {"template_id", "start", "end"?, "params"}` (the
   length defaults to `meta.defaultDuration`; it may run inside `apply_batch`).
3. Always check with `get_frames` at the entrance, middle and exit of the instance.
4. To change values: `update_template_params`. To change the code: read it with
   `list_templates {"template_id"}`, then `update_motion_template` with the whole new module
   (instances outside the task's scope keep the old version: the answer says `copied_from`).

Prefer plain items (`add_text` + presets) for a simple title; use a template when the motion is
procedural or several elements move together.

## The module

```ts
export const meta = { name: 'Lower third', description: 'What it shows', defaultDuration: 5 };

export const params = {
  name: { type: 'string', title: 'Nom', default: 'Jane Doe' },
  accent: { type: 'string', format: 'color', title: 'Accent', default: '#ffcc00' },
};

export function render(ctx, time, params) {
  // draw the frame at `time` (seconds since the instance's start)
}
```

- `meta`: `name` (shown to the user), `description`, `defaultDuration` in seconds.
- `params`: one entry per parameter, in the order the inspector shows them. `title` is the label
  the user reads (write it in French), `description` an optional tooltip. Every parameter needs a
  `default` except assets.

| Schema | The inspector shows | `render` receives |
|---|---|---|
| `{ type: 'string', default, multiline?, maxLength? }` | a text field (area when multiline) | a string |
| `{ type: 'number' \| 'integer', default, minimum?, maximum?, multipleOf? }` | a scrub field with keyframes | a number (animated by the user's keyframes) |
| `{ type: 'string', format: 'color', default: '#rrggbb[aa]' }` | a colour field with keyframes | `'#rrggbbaa'` |
| `{ type: 'boolean', default }` | a switch | `true` / `false` |
| `{ type: 'string', enum: [...], enumLabels?: [...], default }` | a select | one of the values |
| `{ type: 'string', format: 'font', default: 'Inter' }` | a font picker | a family name |
| `{ type: 'string', format: 'asset' }` | an image picker | an image handle `{ width, height }` or `null` |

- `render(ctx, time, params)` draws one frame, **synchronously**. `ctx.duration` is the instance's
  length (the user may trim it): time entrances from 0 and exits from `ctx.duration`.

## Drawing (`ctx`)

The template draws over the **whole frame in design pixels**: `ctx.width` × `ctx.height` (the
shorter side is 1080: 1920 × 1080 in 16:9, 1080 × 1920 in 9:16). Lay things out from `ctx.width`
and `ctx.height`, never assume one format. The frame is transparent: draw only what shows.

- Canvas 2D subset: `fillStyle`, `strokeStyle` (colours or gradients), `lineWidth`, `lineCap`,
  `lineJoin`, `setLineDash`, `globalAlpha`, `globalCompositeOperation` (blend modes: `multiply`,
  `screen`, `overlay`…), `shadowColor`, `shadowBlur`, `shadowOffsetX/Y`, `filter = 'blur(8px)'`,
  `font`, `textAlign`, `textBaseline`, `letterSpacing`; `save`, `restore`, `translate`, `rotate`,
  `scale(x, y?)`, `transform`, `setTransform`, `resetTransform`; paths (`beginPath`, `moveTo`,
  `lineTo`, `bezierCurveTo`, `quadraticCurveTo`, `arc`, `arcTo`, `ellipse`, `rect`, `roundRect`,
  `closePath`, `fill`, `stroke`, `clip`), `ctx.path('M0 0 L10 10 Z')` (SVG path data, for `fill`,
  `stroke`, `clip`), `fillRect`, `strokeRect`, `clearRect`, `fillText`, `strokeText`,
  `measureText`, `createLinearGradient`, `createRadialGradient`, `createConicGradient`
  (`.addColorStop(offset, colour)`), `drawImage(params.logo, x, y, w?, h?)`.
- Text helpers (they leave the context as they found it):
  - `ctx.setFont({ family, size, weight, style })` sets `ctx.font` (family defaults to Inter).
  - `ctx.text(text, x, y, { family, size, weight, style, color, align, valign, maxWidth,
    lineHeight, letterSpacing, stroke: { color, width } })` draws a block wrapped at `maxWidth`
    (and at `\n`), anchored at `(x, y)`: `align` left/center/right, `valign` top/middle/bottom.
    Returns `{ lines, width, height }`.
  - `ctx.layoutText(text, spec)` measures the same block without drawing.
  - Fonts: the families of `font` parameters and any family you name load automatically (from the
    app's catalogue; Inter is always there).
- Helpers: `ctx.ease.linear | inQuad | outQuad | inOutQuad | inCubic | outCubic | inOutCubic |
  inQuart | outQuart | inOutQuart | inExpo | outExpo | inOutExpo | inSine | outSine | inOutSine |
  inBack | outBack | inOutBack | outElastic | inBounce | outBounce` (each takes `t` in 0–1),
  `ctx.progress(time, start, end)` (0 before, 1 after), `ctx.lerp(a, b, t)`,
  `ctx.clamp(v, min, max)`, `ctx.random(seed)` (returns a generator of numbers in [0, 1), the same
  for the same seed), `ctx.mixColor(a, b, t)`, `ctx.withAlpha(colour, alpha)`.

## Rules (the check refuses modules that break them)

- **Deterministic**: the same `time` and `params` always draw the same picture. No `Date`, no
  `Math.random` (use `ctx.random(seed)`), no state kept between calls (module-level counters,
  caches of what was drawn): the editor renders frames in any order, twice, in preview and export.
- **No outside world**: no `fetch`, network, timers, workers, storage, DOM, `eval`, `import` or
  `require`. Everything comes through `ctx`, `time` and `params`. Built-in prototypes are frozen.
- **Budget**: each frame must render in under 50 ms at the design size; stay well below (a few
  milliseconds is normal). No per-pixel loops, few blurs and shadows, measure texts once per frame.
- `render` returns nothing and never awaits.
- TypeScript types are removed but not checked: keep the code simple and plain.

## Good motion

- Entrances 0.3–0.6 s with `outCubic`/`outBack`, exits 0.3–0.5 s with `inCubic`; stagger related
  elements by 0.05–0.15 s; nothing moves for the sake of moving once in place.
- Legibility first: big bold text (names 64–96, roles 40–56, titles 100–160 design pixels), high
  contrast, a plate, outline or shadow over busy video; keep text inside the safe areas (5 % margin
  in 16:9; in 9:16 avoid the top 12 %, the bottom 20 % and the right edge).
- Expose what the user will want to change: every text, the main colours, the font, an offset or a
  speed, a choice of position. Give parameters French titles and sensible ranges.
- One template, one purpose; reuse it with several instances rather than writing near-copies.

## Examples

Four complete modules that pass the check (copy and adapt them):

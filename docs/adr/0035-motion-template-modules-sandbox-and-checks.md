# 0035 — Motion template modules: one source of truth, a sandboxed Worker runtime, checks by the renderer

- Status: Accepted
- Date: 2026-11-09

## Context
ADR-0011 decided that AI agents write motion templates (a TypeScript module drawing with a
restricted 2D API, plus a parameter schema the inspector turns into controls) and that they run
in a sandbox. Prompt 24 built them, which forced several concrete choices: where the parameter
schema lives, who compiles and checks a module (the core has no JavaScript runtime), how far the
sandbox goes inside a renderer that also holds the session token, how frames reach the compositor
without breaking the single code path of preview and export (golden rule 7), and what happens to
existing instances when an agent writes a new version — possibly from a task whose scope does not
cover every instance.

## Decision
1. **The module is the only source of truth.** An agent sends one string: the whole module with
   `export const meta = { name, description, defaultDuration }`, `export const params = { … }` (a
   JSON Schema subset: `string` (multiline, `maxLength`), `number`/`integer` (`minimum`,
   `maximum`, `multipleOf`), `string` + `format: color | font | asset`, `boolean`, `string` +
   `enum` (+ `enumLabels`)) and `export function render(ctx, time, params)`. There is no separate
   `params_schema` argument that could disagree with the code. The renderer evaluates the module
   and reports `meta` and `params` as JSON (params as `[name, schema]` pairs, since object key
   order is the inspector's order and `serde_json` does not keep it); the core reads them
   **strictly** (`se_domain::templates::parse_definition`: names, kinds, defaults, ranges, limits)
   into a typed `MotionTemplate { id, name, description, version, source, params:
   [TemplateParam { name, label, description, spec }], defaultDuration, origin }`. No thumbnail is
   stored: the inspector and the template list render the template live.
2. **The renderer checks, the core decides.** `create_motion_template` and
   `update_motion_template` go through `se_app::TemplateCheckService`: the core publishes
   `templateCheckRequested { requestId, source, width, height, pictureWidth, pictureHeight }` (the
   design frame of the task's sequence), the renderer answers `reportTemplateCheck` (the same
   round trip as `get_frames`, ADR-0028; 45 s timeout, `RENDERER_UNAVAILABLE` without a window).
   `@se/engine/templates` `checkTemplate` compiles (sucrase, embedded: TypeScript types removed,
   lines kept; `import`/`require` refused), loads the module (exports present, 3 s watchdog),
   renders the instants 0, entrance, middle, exit and end with the default values twice — a
   warm-up pass then a timed pass, so first-use costs (fonts, JIT) do not count — compares both
   passes pixel by pixel (determinism) and checks each frame against the **50 ms budget**; a frame
   still running after 1 s kills the Worker (an infinite loop is reported as a budget problem at
   that instant). It answers either `passed` (meta, params, three JPEG pictures, slowest frame) or
   `failed` with every problem: stage (`compile`, `exports`, `render`, `budget`, `determinism`,
   `definition`), message, line and column in the agent's source, instant. The agent gets
   `TEMPLATE_INVALID` listing them all, one per line. **An invalid module never enters the
   project.**
3. **Sandbox: one blob Worker per template version, globals removed, prototypes frozen.** The
   Worker's source is self-contained functions serialized with `toString()` and the compiled
   module wrapped in a function whose parameters shadow `globalThis`, `self`, `window`,
   `postMessage`, `close`, `importScripts`; nothing is declared at the script's top level, so the
   module never sees the runtime's variables. Before the module runs, `neutralizeGlobals` deletes
   the network (`fetch`, `XMLHttpRequest`, `WebSocket`, `EventSource`…), workers and channels,
   storage, timers, `performance`, `crypto`, `Math.random`, `Date`, `OffscreenCanvas`, fonts and
   code generation (`eval`, every `Function` constructor reachable through function prototypes,
   `WebAssembly`) from the whole global prototype chain and puts throwing getters in their place
   (the error names what was used and what to do); built-in prototypes are frozen. The module
   draws on an `OffscreenCanvas` it never touches directly: `ctx` is a facade over a Canvas 2D
   subset in **design pixels** (shorter side 1080; shadows and blurs scaled to the output size),
   text helpers (`setFont`, `text` with wrapping and anchors, `layoutText`), SVG `path`, images
   only through handles of `asset` parameters, and helpers (`ease.*`, `progress`, `lerp`, `clamp`,
   `random(seed)`, `mixColor`, `withAlpha`). A blob Worker inherits the page's CSP (no
   `unsafe-eval`, connections to the local core only) and holds no token: removing the network
   objects is the first barrier, the CSP the second.
4. **The engine renders templates like any other picture.** `evaluate` turns a template item into
   a `TemplateLayer` (values resolved at the item-relative instant: animatable numbers and colours
   interpolated, assets and fonts named, seconds since the start, the instance's length) placed
   with the item's `Transform2D` over a full-frame box. `TemplateRuntime` (one Worker pool per
   runtime, at most 8 Workers, least recently used stopped) renders the layer to an `ImageBitmap`
   cached and leased like decoded pictures (key: template, version, values, size, instant); fonts
   are sent to the Worker as bytes from the catalogue, images as bitmaps. The preview engine, the
   still renderer of `get_frames` and later the export use the same runtime. A failing template
   draws a visible striped error picture and reports a `TemplateIssue` (message, line) the
   inspector shows; the rest of the frame still renders.
5. **Versions follow the caption styles' copy on write (ADR-0033).** `update_motion_template`
   gives the template version `n + 1` **in place** when every instance (before and after) lies in
   the task's scope; otherwise the template is **copied** (new id, the new version) for the
   instances in scope, and the others keep the old one (`copied_from` in the answer). Instance
   values are migrated by name (`se_domain::templates::migrate_params`): a value whose parameter is
   gone, changed kind or is refused by the new range falls back to the default and is reported
   (`reset`). The scope rule refuses any other change of a template used outside the scope.
6. **Instances are ordinary graphics.** `ItemKind::MotionTemplate(TemplateItem { templateId,
   params: name → Text | Number (Animatable) | Boolean | Color (Animatable) | Enum | Font | Asset,
   transform })` takes moves, trims (keyframes follow), splits, presets and the transform gizmo;
   unset values read the template's defaults. Operations `AddTemplate`, `ReplaceTemplate`,
   `RemoveTemplate` (refused while used: `TEMPLATE_IN_USE`) with exact inverses; validation codes
   `INVALID_TEMPLATE`, `INVALID_TEMPLATE_PARAMS`; document schema 9.

## Consequences
- An agent can write any 2D motion graphic and iterate on precise, line-numbered errors; the user
  edits every exposed value in a generated inspector (previewed live, one undo step per gesture,
  numbers and colours keyframable) and never sees the code unless they ask (« Voir le code »).
- Checking needs an editor window: without one, template tools fail with `RENDERER_UNAVAILABLE`
  (like `get_frames`). Headless export (prompt 27) must provide the same runtime.
- The full-frame layer box means a click in the viewer over a template selects it even where it
  draws nothing (backlog: hit-testing on drawn pixels).
- Determinism is checked on a few instants only; a module whose output depends on hidden state at
  other instants could still slip through — the frozen prototypes and missing clocks make that
  hard, and the guide forbids state between calls.
- Template sources are code in `project.json`; they are inert outside the sandbox (never
  evaluated by the core, the main process or the renderer's page).
- A global template library and brand kits remain prompt 25.

## Alternatives considered
- **A separate `params_schema` argument (as ADR-0011 and the prompt sketched)**: two sources that
  can disagree with what `render` reads. Rejected: the module declares its parameters.
- **Running modules in the core (an embedded JS engine)**: a second rendering path (breaks
  WYSIWYG), no Canvas 2D, a large dependency. Rejected; the core only parses the declaration.
- **An iframe or a ShadowRealm sandbox**: an iframe brings a DOM and runs on the page's thread (no
  watchdog without killing the page); ShadowRealm is not available in Electron's Chromium.
  Rejected for Workers, which can be terminated.
- **Changing a template in place for every instance**: lets a task change graphics outside its
  scope. Rejected for copy on write.
- **Storing a rendered thumbnail**: stale after any version and an opaque render in the project
  (golden rule 6). Rejected: rendered live.

# 0036 — Template library and brand kits: app-wide libraries, project copies, proposed updates

- Status: Accepted
- Date: 2026-11-09

## Context
Prompt 25 asks to reuse work across projects: a global library of motion templates (titles,
lower thirds, calls to action, caption styles) and brand kits (fonts, colours, logos, default
looks) that new graphics and AI agents apply. Three constraints shape it: a project must open and
render on another machine without the library (golden rule 1: no network, no shared server); a
newer library version must be **proposed, never imposed** on projects already using a template;
and agents act only through MCP tools (golden rule 2), with the kit used consistently unless the
user asks otherwise.

## Decision
1. **App-wide libraries, project-wide copies.** Library entries live in the app data folder:
   `templates/<id>/` (`template.json` — the entry without its module, with a `formatVersion` —
   and `source.ts` for motion templates; a caption style entry holds its look) and
   `brand-kits/<id>/` (`brand-kit.json` and the kit's own copies of its logos under `logos/`).
   se-store implements them (`FsTemplateLibrary`, `FsBrandKitLibrary`) behind se-app ports
   (`TemplateLibrary`, `BrandKitLibrary`); removals go to the system trash. A project never
   refers to the library for anything it renders: inserting a library template **copies** it
   into `Project.templates` (`MotionTemplate.library = { templateId, version }` records where it
   came from), and choosing a kit **copies** it into `Project.brandKit`
   (`source = { kitId, version }`), its logos copied into `<project>/brand-kit/<kit>/` and
   imported as image assets. Document schema 10.
2. **Versions are integers; updates are offered, then applied on request.** Saving a project
   template to the library makes a new entry, or — when the template came from an entry — a new
   version of it (only if its module changed). A project copy older than the library's version
   shows « vN disponible »; « Mettre à jour » replaces the copy (`ReplaceTemplate`, the project's
   own version + 1) and migrates instance values by name (`se_domain::migrate_params`, values the
   new parameters refuse reset), in one undoable step. Inserting the same library version twice
   reuses the existing copy (same entry, version and source). An agent's new version of a linked
   copy keeps its link, so it can be saved back as a new library version.
3. **Brand kits follow the open project at once, others on request.** Saving a library kit (a new
   version) updates the copy of every project open in the core that uses it (one undoable step
   « Update brand kit » each); a project opened later with an older copy shows the newer version
   in its header menu. A kit absent from the library (made on another machine) can be added back.
4. **The kit is a default, applied where things are made.** New texts take the kit's text style
   (the user's « Texte », agents' `add_text` fields left out); new caption styles take its caption
   look (`create_captions`/`generateCaptions` without a style, or the `brandKit` choice); new
   template instances take the kit's value for parameters bound to a **brand role** — `brand:
   'primary' | 'secondary' | 'accent' | 'text' | 'background' | 'heading' | 'body' | 'logo'` in
   the module's parameter schema (`se_domain::brand_values`), values given explicitly winning. The
   values become ordinary data of what they styled: changing the kit later restyles nothing
   already made. Every colour and font field of the editor offers the kit's palette and fonts as
   shortcuts (`ColorField` named swatches, the font menu's first group).
5. **Agents read the kit, list the library, insert from it.** `get_brand_kit` answers the copy;
   the task's instructions summarise it in one line (fonts, palette, logos) and ask for consistent
   use unless the user says otherwise (`09-brand-kit.md`); `list_templates { source: "library",
   category }` lists library motion templates (with brand roles and the project's copy);
   `insert_library_template` copies and places one (`AgentEdit::InsertLibraryTemplate`). Agents
   cannot write to the library: saving is the user's decision. Modules may declare
   `meta.category` (`title`, `lowerThird`, `callToAction`, `transition`, `other`).
6. **`.setemplate` exchange files** are zips of the same `template.json` and `source.ts`. An
   import runs the motion module through the same check as an agent's (`TemplateCheckService`:
   the renderer compiles, renders, measures, compares; the domain reads the declarations, which
   replace whatever the archive claims); a caption style is checked by `CaptionStyle::problems`.
   A newer version of an existing entry replaces it; an equal or older one changes nothing.
7. **No stored thumbnails.** As in ADR-0035, the browser renders templates live with the window's
   runtime (animated on hover); caption styles are drawn on a sample line.

## Consequences
- Projects stay self-contained: the library is a convenience for making and updating copies, never
  a dependency (moving a project to another machine loses only the update notices and favourites'
  insertion, which need the library).
- Brand kit logos are copied twice (into the kit, then into each project using it): a little disk
  space for portability; the user's original image is never touched.
- A kit's caption look and text style hold static values only (validated: `INVALID_BRAND_KIT`).
- Brand roles let one library template fit any brand, but only at creation: an instance keeps the
  values it started with.
- Imports need an editor window (the renderer checks the module), like template tools.
- The library has no history of versions: an entry keeps its latest content and version number.

## Alternatives considered
- **Projects referencing library entries** (no copies): breaks opening on another machine and lets
  a library change silently restyle old projects. Rejected.
- **Automatic updates of project copies**: the prompt asks for proposals; a template change could
  break an edit the user was happy with. Rejected (except the open project of an edited kit, which
  is the user's own gesture on that kit).
- **Live brand binding** (instances reading the kit at render time): restyles finished edits when
  a kit changes and hides values from the inspector. Rejected for values copied at creation.
- **A SQLite table for the library**: entries are small files the user may back up or move; a
  folder per entry with JSON and the module keeps them readable and diffable. Rejected.
- **Stored `thumbnail.png`** (as the prompt sketched): stale after any version; live previews
  already exist. Rejected.

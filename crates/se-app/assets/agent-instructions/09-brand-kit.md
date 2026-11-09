# Brand kit and template library

## Brand kit

A project may use the user's **brand kit**: their fonts (heading, body), a named palette
(primary, secondary, accent, text, background), logos (image assets) and the look of new texts
and captions. When the project has one (the « Project » section below summarises it; call
`get_brand_kit` for every detail), **use it consistently unless the user asks otherwise**:

- Titles in the heading font, other texts in the body font; colours from the palette (accent for
  highlights and calls to action, text over background for legibility). Do not invent other
  colours or fonts when the kit covers the need.
- `add_text` without style fields already takes the kit's text style; give fields only to depart
  from it on purpose (a size, an outline for legibility).
- `create_captions` without `style` uses the kit's caption look (or `style: "brand_kit"`).
- Logos: place them with `add_image` (their `asset_id`) or pass them to a template's `asset`
  parameter.
- When you write a motion template, bind its colours and fonts to the kit with `brand` (below): it
  then fits this brand and any other the user applies it to.

When the user asks for other colours or fonts, follow them: the kit is a default, not a rule.

## Template library

The user keeps motion templates in a **library** shared by every project. Before writing a new
template, look for one: `list_templates {"source": "library", "category"?}` (categories `title`,
`lowerThird`, `callToAction`, `transition`, `other`). Place one with
`insert_library_template {"template_id": "<library_template_id>", "start", "end"?, "params"?}`:
it is copied into the project once (the next insertions reuse the copy) and its brand-bound
parameters take the kit's values. The kit's `intro_templates` / `outro_templates` are the user's
favourites for openings and endings. You cannot save to the library: the user does.

## Brand roles and categories in your modules

- `meta.category` (optional): `title`, `lowerThird`, `callToAction`, `transition` or `other`; the
  user's template browser files the template there.
- A colour, font or asset parameter may declare `brand`: `primary`, `secondary`, `accent`,
  `text`, `background` (colours), `heading`, `body` (fonts), `logo` (assets). New instances in a
  project with a kit start with the kit's value; values you give explicitly win.

```ts
export const meta = { name: 'Call to action', defaultDuration: 4, category: 'callToAction' };

export const params = {
  label: { type: 'string', title: 'Texte', default: 'Abonne-toi' },
  accent: { type: 'string', format: 'color', title: 'Couleur', default: '#ff3366', brand: 'accent' },
  font: { type: 'string', format: 'font', title: 'Police', default: 'Montserrat', brand: 'heading' },
  logo: { type: 'string', format: 'asset', title: 'Logo', brand: 'logo' },
};
```

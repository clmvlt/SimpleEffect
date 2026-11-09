// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.

/**
 * A role of a brand kit: a palette colour, a font or the main logo.
 *
 * A motion template parameter declares one (`brand: 'accent'` in its schema) so instances made
 * in a project with a kit start with the kit's value instead of the schema default: one library
 * template then fits every brand.
 */
export type BrandRole = "primary" | "secondary" | "accent" | "text" | "background" | "heading" | "body" | "logo";

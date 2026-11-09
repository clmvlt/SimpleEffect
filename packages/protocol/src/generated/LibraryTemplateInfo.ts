// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { TemplateCategory } from "./TemplateCategory";

/**
 * What « Enregistrer dans la bibliothèque » keeps of a project template or caption style.
 */
export type LibraryTemplateInfo = { 
/**
 * Name shown in the browser (not empty).
 */
name: string, 
/**
 * What it shows.
 */
description: string, 
/**
 * Free words to search by.
 */
tags: Array<string>, 
/**
 * Browser category of a motion template (ignored for caption styles; `null` keeps the
 * template's own).
 */
category?: TemplateCategory, };

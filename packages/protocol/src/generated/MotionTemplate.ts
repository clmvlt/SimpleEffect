// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Flicks } from "./Flicks";
import type { LibraryLink } from "./LibraryLink";
import type { Origin } from "./Origin";
import type { TemplateCategory } from "./TemplateCategory";
import type { TemplateId } from "./TemplateId";
import type { TemplateParam } from "./TemplateParam";

/**
 * An AI-authored, parameterised render module stored in the project.
 *
 * The TypeScript `source` is the single truth (ADR-0035): it exports `meta` (name, description,
 * default duration), `params` (the parameter schema) and `render(ctx, time, params)`. The
 * renderer compiles and runs it in a sandboxed Worker; the core keeps what `meta` and `params`
 * declared, checked, in the other fields. A template is added only once it compiled, rendered
 * within its budget and proved deterministic; a new source makes a new `version`.
 */
export type MotionTemplate = { 
/**
 * Stable id, referenced by template items.
 */
id: TemplateId, 
/**
 * Name shown in the library and the inspector (`meta.name`).
 */
name: string, 
/**
 * What it shows (`meta.description`; may be empty).
 */
description: string, 
/**
 * Version of the source, 1 for a new template, bumped by each new source.
 */
version: number, 
/**
 * The TypeScript module (`meta`, `params`, `render`).
 */
source: string, 
/**
 * The parameters `params` declares, in declaration order.
 */
params: Array<TemplateParam>, 
/**
 * Length of new instances (`meta.defaultDuration`); > 0.
 */
defaultDuration: Flicks, 
/**
 * Who wrote it.
 */
origin: Origin, 
/**
 * Where the template browser files it (`meta.category`; schema version 10).
 */
category: TemplateCategory, 
/**
 * The app's library entry it was copied from, if any (schema version 10).
 */
library: LibraryLink | null, };

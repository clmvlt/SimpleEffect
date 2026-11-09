// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { LibraryContent } from "./LibraryContent";
import type { LibraryTemplateId } from "./LibraryTemplateId";

/**
 * One entry of the template library.
 */
export type LibraryTemplate = { 
/**
 * Stable id (project copies link to it).
 */
id: LibraryTemplateId, 
/**
 * Version of the content, 1 when saved first, bumped by each new content; project copies
 * holding an older version are offered the update.
 */
version: number, 
/**
 * Name shown in the browser.
 */
name: string, 
/**
 * What it shows (may be empty).
 */
description: string, 
/**
 * Free words to search by.
 */
tags: Array<string>, 
/**
 * When it last changed (Unix milliseconds).
 */
updatedAt: number, 
/**
 * The template itself.
 */
content: LibraryContent, };

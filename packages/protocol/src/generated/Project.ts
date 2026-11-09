// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Asset } from "./Asset";
import type { BrandKit } from "./BrandKit";
import type { CaptionStyle } from "./CaptionStyle";
import type { MotionTemplate } from "./MotionTemplate";
import type { ProjectId } from "./ProjectId";
import type { ProjectSettings } from "./ProjectSettings";
import type { Sequence } from "./Sequence";

/**
 * The whole editable state of a project, serialized as `project.json` (ADR-0006).
 *
 * Mutated only by the project actor applying operations (golden rule 4, prompt 06).
 */
export type Project = { 
/**
 * Stable id.
 */
id: ProjectId, 
/**
 * Name shown in the UI (also the default folder name).
 */
name: string, 
/**
 * Document format version ([`CURRENT_SCHEMA_VERSION`] once loaded).
 */
schemaVersion: number, 
/**
 * Number of transactions committed so far; bumped by every commit.
 */
revision: number, 
/**
 * Project-wide preferences.
 */
settings: ProjectSettings, 
/**
 * Imported media, in import order.
 */
assets: Array<Asset>, 
/**
 * Timelines, in creation order.
 */
sequences: Array<Sequence>, 
/**
 * Motion templates available to template items.
 */
templates: Array<MotionTemplate>, 
/**
 * The brand identity used by this project, if one was set: a copy of a library kit
 * (schema version 10), so the project behaves the same without the library.
 */
brandKit: BrandKit | null, 
/**
 * Looks shared by its captions (schema version 8).
 */
captionStyles: Array<CaptionStyle>, };

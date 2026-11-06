// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { ItemId } from "./ItemId";
import type { ItemKind } from "./ItemKind";
import type { LinkGroupId } from "./LinkGroupId";
import type { Origin } from "./Origin";
import type { TimeRange } from "./TimeRange";

/**
 * Something placed on a track over a time range: a media clip, a text, a template…
 */
export type Item = { 
/**
 * Stable id, unique in the project.
 */
id: ItemId, 
/**
 * Bumped on every change to this item; agents send it back for optimistic concurrency.
 */
revision: number, 
/**
 * Who created the item, kept for traceability and changeset reverts.
 */
origin: Origin, 
/**
 * Where the item sits in its sequence; never empty, never before zero.
 */
timelineRange: TimeRange, 
/**
 * Items sharing a link group move and trim together (a clip's picture and sound).
 */
linkGroup: LinkGroupId | null, 
/**
 * What the item shows or plays.
 */
kind: ItemKind, };

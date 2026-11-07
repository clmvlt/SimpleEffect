// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { CaptionPatch } from "./CaptionPatch";
import type { ImagePatch } from "./ImagePatch";
import type { MediaClipPatch } from "./MediaClipPatch";
import type { ShapePatch } from "./ShapePatch";
import type { TemplatePatch } from "./TemplatePatch";
import type { TextPatch } from "./TextPatch";

/**
 * A change of an item's content; its kind must match the item's.
 */
export type ItemPatch = { "kind": "mediaClip" } & MediaClipPatch | { "kind": "text" } & TextPatch | { "kind": "shape" } & ShapePatch | { "kind": "image" } & ImagePatch | { "kind": "motionTemplate" } & TemplatePatch | { "kind": "caption" } & CaptionPatch;

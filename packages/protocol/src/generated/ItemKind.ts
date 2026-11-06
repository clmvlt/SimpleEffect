// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { CaptionItem } from "./CaptionItem";
import type { ImageItem } from "./ImageItem";
import type { MediaClip } from "./MediaClip";
import type { ShapeItem } from "./ShapeItem";
import type { TemplateItem } from "./TemplateItem";
import type { TextItem } from "./TextItem";

/**
 * What an item shows or plays. Serialized with its fields next to `"kind"`.
 *
 * Only media clips may sit on audio tracks (playing the sound of their asset); every kind may
 * sit on a video track except a clip of an audio-only asset.
 */
export type ItemKind = { "kind": "mediaClip" } & MediaClip | { "kind": "text" } & TextItem | { "kind": "shape" } & ShapeItem | { "kind": "image" } & ImageItem | { "kind": "motionTemplate" } & TemplateItem | { "kind": "caption" } & CaptionItem;

// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { CaptionStyle } from "./CaptionStyle";
import type { Flicks } from "./Flicks";
import type { TemplateCategory } from "./TemplateCategory";
import type { TemplateParam } from "./TemplateParam";

/**
 * What a library entry holds.
 */
export type LibraryContent = { "kind": "motionTemplate", 
/**
 * Its browser category.
 */
category: TemplateCategory, 
/**
 * The TypeScript module.
 */
source: string, 
/**
 * The parameters it declares, in order.
 */
params: Array<TemplateParam>, 
/**
 * Length of new instances.
 */
defaultDuration: Flicks, } | { "kind": "captionStyle", 
/**
 * The look.
 */
style: CaptionStyle, };

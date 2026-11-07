// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { TemplateId } from "./TemplateId";
import type { TemplateParamValue } from "./TemplateParamValue";

/**
 * Changes of a [`crate::TemplateItem`].
 */
export type TemplatePatch = { 
/**
 * Another template.
 */
templateId: TemplateId | null, 
/**
 * Parameter values to set (a value) or reset to the template default (`null`); parameters
 * not listed are unchanged.
 */
params: { [key in string]: TemplateParamValue | null }, };

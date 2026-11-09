// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { TemplateId } from "./TemplateId";
import type { TemplateParamValue } from "./TemplateParamValue";
import type { Transform2D } from "./Transform2D";

/**
 * An item rendering a motion template with parameter values.
 */
export type TemplateItem = { 
/**
 * The template rendered (must exist in the project).
 */
templateId: TemplateId, 
/**
 * Parameter values by parameter name; a name missing here takes the schema default.
 */
params: { [key in string]: TemplateParamValue }, 
/**
 * Placement of the template's frame (it draws over the whole frame, then is placed like any
 * graphic: position, scale, rotation, opacity, blur). Centred and natural by default.
 */
transform: Transform2D, };

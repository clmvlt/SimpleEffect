// GENERATED — DO NOT EDIT. Source: Rust types deriving `ts_rs::TS` (crates/se-protocol,
// crates/se-domain). Regenerate with `pnpm codegen`.
import type { Animatable } from "./Animatable";
import type { Color } from "./Color";
import type { DesignPixels } from "./DesignPixels";
import type { Vec2 } from "./Vec2";

/**
 * An animatable property with its new value, typed by property (positions are
 * [`Vec2`], rotations numbers, sizes design pixels, colours [`Color`]). Serialized as
 * `{"property": "opacity", "value": {"kind": "keyframed", …}}`.
 */
export type PropertyAnimation = { "property": "position", 
/**
 * Static value or keyframes.
 */
value: Animatable<Vec2>, } | { "property": "scale", 
/**
 * Static value or keyframes.
 */
value: Animatable<Vec2>, } | { "property": "rotation", 
/**
 * Static value or keyframes.
 */
value: Animatable<number>, } | { "property": "anchor", 
/**
 * Static value or keyframes.
 */
value: Animatable<Vec2>, } | { "property": "opacity", 
/**
 * Static value or keyframes.
 */
value: Animatable<number>, } | { "property": "blur", 
/**
 * Static value or keyframes.
 */
value: Animatable<DesignPixels>, } | { "property": "volumeDb", 
/**
 * Static value or keyframes.
 */
value: Animatable<number>, } | { "property": "reframeFocus", 
/**
 * Static value or keyframes.
 */
value: Animatable<Vec2>, } | { "property": "fontSize", 
/**
 * Static value or keyframes.
 */
value: Animatable<DesignPixels>, } | { "property": "fillColor", 
/**
 * Static value or keyframes.
 */
value: Animatable<Color>, } | { "property": "letterSpacing", 
/**
 * Static value or keyframes.
 */
value: Animatable<DesignPixels>, } | { "property": "reveal", 
/**
 * Static value or keyframes.
 */
value: Animatable<number>, } | { "property": "strokeColor", 
/**
 * Static value or keyframes.
 */
value: Animatable<Color>, } | { "property": "strokeWidth", 
/**
 * Static value or keyframes.
 */
value: Animatable<DesignPixels>, } | { "property": "shadowColor", 
/**
 * Static value or keyframes.
 */
value: Animatable<Color>, } | { "property": "shadowBlur", 
/**
 * Static value or keyframes.
 */
value: Animatable<DesignPixels>, } | { "property": "shadowOffset", 
/**
 * Static value or keyframes.
 */
value: Animatable<Vec2>, } | { "property": "backgroundColor", 
/**
 * Static value or keyframes.
 */
value: Animatable<Color>, };

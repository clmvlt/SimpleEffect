/**
 * Keyframe interpolation shared by preview and export: easing curves (linear, hold, cubic Bézier,
 * presets) and blending of numbers, vectors and colours (OKLab).
 */
export { cubicBezierProgress, type CubicBezierCurve } from './cubic-bezier';
export { EASING_PRESET_CURVES, easedProgress } from './easing';
export {
  BLACK,
  formatColor,
  mixColors,
  oklabToRgba,
  parseColor,
  rgbaToOklab,
  type OklabColor,
  type Rgba,
} from './color';
export {
  interpolateColor,
  interpolateNumber,
  interpolateVec2,
  keyframesValueAt,
  valueAt,
  type Interpolator,
} from './interpolate';

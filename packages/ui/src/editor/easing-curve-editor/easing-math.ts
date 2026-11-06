/**
 * An easing between two keyframes: a CSS-style cubic Bézier from (0, 0) to (1, 1) with control
 * points (x1, y1) and (x2, y2), or `hold` (the value jumps at the next keyframe). Presentational
 * type of `EasingCurveEditor`; the app maps it to the domain's interpolation type.
 */
export type EasingCurve =
  | {
      readonly kind: 'cubicBezier';
      readonly x1: number;
      readonly y1: number;
      readonly x2: number;
      readonly y2: number;
    }
  | { readonly kind: 'hold' };

/** Presets offered by the editor. */
export type EasingPresetId = 'linear' | 'easeIn' | 'easeOut' | 'easeInOut' | 'back' | 'hold';

/** The cubic Bézier member of {@link EasingCurve}. */
export type CubicBezierCurve = Extract<EasingCurve, { kind: 'cubicBezier' }>;

/** Builds a cubic Bézier curve. */
export function cubicBezier(x1: number, y1: number, x2: number, y2: number): CubicBezierCurve {
  return { kind: 'cubicBezier', x1, y1, x2, y2 };
}

/** The preset curves (CSS keyword values; `back` overshoots then settles). */
export const EASING_PRESETS: Readonly<Record<EasingPresetId, EasingCurve>> = {
  linear: cubicBezier(0, 0, 1, 1),
  easeIn: cubicBezier(0.42, 0, 1, 1),
  easeOut: cubicBezier(0, 0, 0.58, 1),
  easeInOut: cubicBezier(0.42, 0, 0.58, 1),
  back: cubicBezier(0.34, 1.56, 0.64, 1),
  hold: { kind: 'hold' },
};

/** Vertical room for overshoot: control points may go below 0 and above 1. */
export const HANDLE_Y_RANGE = { min: -0.6, max: 1.6 } as const;

/** Size of the SVG drawing (viewBox units) and its inner padding. */
export const PLOT = { size: 100, padding: 8 } as const;

const PLOT_SPAN = PLOT.size - 2 * PLOT.padding;
const Y_SPAN = HANDLE_Y_RANGE.max - HANDLE_Y_RANGE.min;

/** A control point of the curve. */
export interface CurvePoint {
  readonly x: number;
  readonly y: number;
}

/** Keeps a control point inside the editable area (x in 0–1 as CSS requires). */
export function clampHandle({ x, y }: CurvePoint): CurvePoint {
  const round = (value: number) => Math.round(value * 1000) / 1000;
  return {
    x: round(Math.min(1, Math.max(0, x))),
    y: round(Math.min(HANDLE_Y_RANGE.max, Math.max(HANDLE_Y_RANGE.min, y))),
  };
}

/** Curve coordinates → SVG viewBox coordinates (y grows downward in SVG). */
export function toPlot({ x, y }: CurvePoint): CurvePoint {
  return {
    x: PLOT.padding + x * PLOT_SPAN,
    y: PLOT.padding + ((HANDLE_Y_RANGE.max - y) / Y_SPAN) * PLOT_SPAN,
  };
}

/** A pointer position over the SVG (whose box is `rect`) → curve coordinates, clamped. */
export function fromClientPoint(
  clientX: number,
  clientY: number,
  rect: Pick<DOMRect, 'left' | 'top' | 'width' | 'height'>,
): CurvePoint {
  const plotX = ((clientX - rect.left) / rect.width) * PLOT.size;
  const plotY = ((clientY - rect.top) / rect.height) * PLOT.size;
  return clampHandle({
    x: (plotX - PLOT.padding) / PLOT_SPAN,
    y: HANDLE_Y_RANGE.max - ((plotY - PLOT.padding) / PLOT_SPAN) * Y_SPAN,
  });
}

/** The preset a curve equals (within rounding), if any. */
export function matchEasingPreset(curve: EasingCurve): EasingPresetId | undefined {
  return (Object.keys(EASING_PRESETS) as EasingPresetId[]).find((id) => {
    const preset = EASING_PRESETS[id];
    if (preset.kind !== curve.kind) {
      return false;
    }
    if (preset.kind === 'hold' || curve.kind === 'hold') {
      return true;
    }
    const close = (a: number, b: number) => Math.abs(a - b) < 1e-3;
    return (
      close(preset.x1, curve.x1) &&
      close(preset.y1, curve.y1) &&
      close(preset.x2, curve.x2) &&
      close(preset.y2, curve.y2)
    );
  });
}

/** CSS `animation-timing-function` of a curve (for the animated preview). */
export function toCssTimingFunction(curve: EasingCurve): string {
  return curve.kind === 'hold'
    ? 'steps(1, end)'
    : `cubic-bezier(${[curve.x1, curve.y1, curve.x2, curve.y2].map(String).join(', ')})`;
}

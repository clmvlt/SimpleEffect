/**
 * A CSS-style cubic Bézier timing curve from `(0, 0)` to `(1, 1)` with control points
 * `(x1, y1)` and `(x2, y2)`; `x1` and `x2` are in `[0, 1]` (validated by the domain), so the
 * curve's x is monotonic and every progress has exactly one parameter.
 */
export interface CubicBezierCurve {
  readonly x1: number;
  readonly y1: number;
  readonly x2: number;
  readonly y2: number;
}

/** Precision of the solved parameter: far below one flick over any keyframe span in use. */
const PARAMETER_EPSILON = 1e-12;
const NEWTON_ITERATIONS = 8;
const BISECTION_ITERATIONS = 64;

/** One coordinate of the curve at parameter `t` (Bernstein form with fixed end points 0 and 1). */
function coordinate(t: number, p1: number, p2: number): number {
  const inverse = 1 - t;
  return 3 * inverse * inverse * t * p1 + 3 * inverse * t * t * p2 + t * t * t;
}

/** Derivative of {@link coordinate} with respect to `t`. */
function slope(t: number, p1: number, p2: number): number {
  const inverse = 1 - t;
  return 3 * inverse * inverse * p1 + 6 * inverse * t * (p2 - p1) + 3 * t * t * (1 - p2);
}

/**
 * The parameter whose x is `x`: Newton–Raphson from `x` itself (fast on smooth curves), then
 * bisection when the slope vanishes or Newton leaves `[0, 1]` (robust on steep and flat curves).
 */
function solveParameter(x: number, curve: CubicBezierCurve): number {
  let t = x;
  for (let iteration = 0; iteration < NEWTON_ITERATIONS; iteration += 1) {
    const error = coordinate(t, curve.x1, curve.x2) - x;
    if (Math.abs(error) < PARAMETER_EPSILON) return t;
    const derivative = slope(t, curve.x1, curve.x2);
    if (Math.abs(derivative) < 1e-9) break;
    const next = t - error / derivative;
    if (next < 0 || next > 1) break;
    t = next;
  }
  let low = 0;
  let high = 1;
  t = x;
  for (let iteration = 0; iteration < BISECTION_ITERATIONS; iteration += 1) {
    const value = coordinate(t, curve.x1, curve.x2);
    if (Math.abs(value - x) < PARAMETER_EPSILON) return t;
    if (value < x) low = t;
    else high = t;
    t = (low + high) / 2;
  }
  return t;
}

/**
 * Eased progress of the curve at linear progress `x` (clamped to `[0, 1]`). The result may leave
 * `[0, 1]` when `y1` or `y2` do (overshoot, e.g. "back" curves).
 */
export function cubicBezierProgress(curve: CubicBezierCurve, x: number): number {
  if (x <= 0) return 0;
  if (x >= 1) return 1;
  return coordinate(solveParameter(x, curve), curve.y1, curve.y2);
}

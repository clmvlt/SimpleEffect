import type { Easing, EasingPreset } from '@se/protocol';

import { cubicBezierProgress, type CubicBezierCurve } from './cubic-bezier';

/**
 * The exact shape of each easing preset (the domain names them, the engine owns them). They are
 * the CSS `ease-in`, `ease-out` and `ease-in-out` curves, the same ones `@se/ui`'s
 * `EasingCurveEditor` offers, so a preset picked there looks identical here.
 */
export const EASING_PRESET_CURVES: Readonly<Record<EasingPreset, CubicBezierCurve>> = {
  easeIn: { x1: 0.42, y1: 0, x2: 1, y2: 1 },
  easeOut: { x1: 0, y1: 0, x2: 0.58, y2: 1 },
  easeInOut: { x1: 0.42, y1: 0, x2: 0.58, y2: 1 },
};

/**
 * Eased progress for a linear progress `progress` in `[0, 1]` between two keyframes:
 * `linear` keeps it, `hold` stays at 0 until the next keyframe, a Bézier or a preset reshapes it.
 */
export function easedProgress(easing: Easing, progress: number): number {
  switch (easing.kind) {
    case 'linear':
      return progress;
    case 'hold':
      return 0;
    case 'cubicBezier':
      return cubicBezierProgress(easing, progress);
    case 'preset':
      return cubicBezierProgress(EASING_PRESET_CURVES[easing.name], progress);
  }
}

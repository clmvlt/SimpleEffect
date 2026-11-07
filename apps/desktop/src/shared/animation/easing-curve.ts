import type { Easing } from '@se/protocol';
import { EASING_PRESET_CURVES } from '@se/engine/animation';
import type { EasingCurve } from '@se/ui';

const LINEAR: EasingCurve = { kind: 'cubicBezier', x1: 0, y1: 0, x2: 1, y2: 1 };

/** The `EasingCurveEditor` curve of a domain easing (presets as their CSS curves). */
export function easingToCurve(easing: Easing): EasingCurve {
  switch (easing.kind) {
    case 'linear':
      return LINEAR;
    case 'hold':
      return { kind: 'hold' };
    case 'cubicBezier':
      return { kind: 'cubicBezier', x1: easing.x1, y1: easing.y1, x2: easing.x2, y2: easing.y2 };
    case 'preset':
      return { kind: 'cubicBezier', ...EASING_PRESET_CURVES[easing.name] };
  }
}

/** The domain easing of an editor curve (the straight curve is `linear`). */
export function curveToEasing(curve: EasingCurve): Easing {
  if (curve.kind === 'hold') return { kind: 'hold' };
  const { x1, y1, x2, y2 } = curve;
  if (x1 === y1 && x2 === y2 && x1 === 0 && x2 === 1) return { kind: 'linear' };
  return { kind: 'cubicBezier', x1, y1, x2, y2 };
}

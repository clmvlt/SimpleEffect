import type { AnimationPresetSample, Flicks, PropertyAnimation } from '@se/protocol';

import { interpolatorOf, valueAtTime } from '../../../shared/animation';

/** CSS of the preview swatch at one instant of a preset's sample. */
export interface PreviewStyle {
  readonly transform: string;
  readonly opacity: number;
  readonly filter: string;
  /** Hides the part of « Aa » not typed yet (`inset(0 x 0 0)`). */
  readonly clipPath: string;
}

/** Size of the swatch the sample's frame maps to, in CSS pixels. */
export interface SwatchSize {
  readonly width: number;
  readonly height: number;
}

/** Design pixels of the sample's frame per CSS pixel of a swatch 54 px high (1080 / 54). */
const DESIGN_PER_PIXEL = 20;

function valueOf(animation: PropertyAnimation, time: Flicks): unknown {
  // Each variant's value and interpolator share the property's value type.
  const value = animation.value as Parameters<typeof valueAtTime<unknown>>[0];
  const interpolate = interpolatorOf(animation.property) as Parameters<
    typeof valueAtTime<unknown>
  >[2];
  return valueAtTime(value, time, interpolate);
}

/**
 * How the swatch looks at `time` (flicks into the one-second sample): the sample animates a
 * centred text in a 1080p frame mapped onto the swatch. The UI never expands presets itself: the
 * sample comes from the core (`listAnimationPresets`).
 */
export function previewStyle(
  sample: AnimationPresetSample,
  time: Flicks,
  swatch: SwatchSize,
): PreviewStyle {
  let x = 0;
  let y = 0;
  let scaleX = 1;
  let scaleY = 1;
  let rotation = 0;
  let opacity = 1;
  let blur = 0;
  let reveal = 1;
  for (const animation of sample.animations) {
    const value = valueOf(animation, time);
    switch (animation.property) {
      case 'position': {
        const point = value as { x: number; y: number };
        x = (point.x - 0.5) * swatch.width;
        y = (point.y - 0.5) * swatch.height;
        break;
      }
      case 'scale': {
        const point = value as { x: number; y: number };
        scaleX = point.x;
        scaleY = point.y;
        break;
      }
      case 'rotation':
        rotation = value as number;
        break;
      case 'opacity':
        opacity = value as number;
        break;
      case 'blur':
        blur = (value as number) / DESIGN_PER_PIXEL;
        break;
      case 'reveal':
        reveal = value as number;
        break;
      default:
        break;
    }
  }
  const round = (value: number) => Math.round(value * 1000) / 1000;
  return {
    transform: `translate(${String(round(x))}px, ${String(round(y))}px) rotate(${String(round(rotation))}deg) scale(${String(round(scaleX))}, ${String(round(scaleY))})`,
    opacity: Math.min(1, Math.max(0, round(opacity))),
    filter: blur > 0 ? `blur(${String(round(blur))}px)` : 'none',
    clipPath: `inset(0 ${String(round((1 - Math.min(1, Math.max(0, reveal))) * 100))}% 0 0)`,
  };
}

import type { Animatable, Asset, Format, ItemKind, Reframe, Vec2 } from '@se/protocol';
import {
  CENTRE_FOCUS,
  assetDisplaySize,
  designScale,
  reframeScale,
  type ReframeMode,
} from '@se/engine/evaluate';

import type { ItemEdit } from '../hooks/use-property-editor';

/** Reframe modes the user picks (tracked focuses come from automatic reframing, prompt 26). */
export const PICKABLE_REFRAME_MODES = ['fit', 'fill', 'custom'] as const;

/** A mode the user picks. */
export type PickableReframeMode = (typeof PICKABLE_REFRAME_MODES)[number];

/** `mode` as a domain value; a fill keeps the focus a fill or tracked clip already has. */
function reframeFor(mode: PickableReframeMode, current: Reframe): Reframe {
  if (mode !== 'fill') return { kind: mode };
  const focus =
    current.kind === 'fill' || current.kind === 'tracked'
      ? current.focus
      : ({ kind: 'static', value: CENTRE_FOCUS } satisfies Animatable<Vec2>);
  return { kind: 'fill', focus };
}

/** `scale` multiplied by `factor` (every keyframe of a keyframed scale). */
function scaled(scale: Animatable<Vec2>, factor: number): Animatable<Vec2> {
  const times = (value: Vec2): Vec2 => ({ x: value.x * factor, y: value.y * factor });
  return scale.kind === 'static'
    ? { kind: 'static', value: times(scale.value) }
    : {
        kind: 'keyframed',
        keyframes: scale.keyframes.map((keyframe) => ({
          ...keyframe,
          value: times(keyframe.value),
        })),
      };
}

/**
 * The edit giving a clip the reframe mode `mode` (`null` when it has it, or is not a clip).
 * Switching to or from `custom` (the natural size) multiplies the clip's scale so the picture
 * keeps its size on screen; switching between fit and fill keeps the scale (a fill fills).
 */
export function reframeEdit(
  kind: ItemKind,
  mode: PickableReframeMode,
  asset: Asset | undefined,
  format: Pick<Format, 'width' | 'height'>,
): ItemEdit | null {
  if (kind.kind !== 'mediaClip' || kind.reframe.kind === mode) return null;
  const patch = { kind: 'mediaClip' as const, reframe: reframeFor(mode, kind.reframe) };
  const current: ReframeMode = kind.reframe.kind;
  if ((current === 'custom') === (mode === 'custom') || asset === undefined) {
    return { patch, animations: [] };
  }
  const display = assetDisplaySize(asset);
  const scale = designScale(format);
  const before = reframeScale(display, format, current, scale);
  const after = reframeScale(display, format, mode, scale);
  if (before === null || after === null || after <= 0) return { patch, animations: [] };
  const factor = before / after;
  return {
    patch,
    animations: [{ property: 'scale', value: scaled(kind.transform.scale, factor) }],
  };
}

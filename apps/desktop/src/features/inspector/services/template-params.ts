import { interpolateColor, interpolateNumber } from '@se/engine';
import type {
  Animatable,
  Color,
  Flicks,
  ItemEntity,
  MotionTemplate,
  TemplateParam,
  TemplateParamValue,
} from '@se/protocol';

import {
  keyframeState,
  neighbourKeyframe,
  toggledKeyframe,
  valueAtTime,
  withValueAt,
  type KeyframeState,
} from '../../../shared/animation';

/** A template instance's values by parameter name. */
export type ParamValues = Readonly<Record<string, TemplateParamValue | undefined>>;

/** The value kind matching each parameter kind (`string` parameters hold `text` values). */
const VALUE_KIND = {
  string: 'text',
  number: 'number',
  color: 'color',
  boolean: 'boolean',
  enum: 'enum',
  font: 'font',
  asset: 'asset',
} as const satisfies Record<TemplateParam['spec']['kind'], TemplateParamValue['kind']>;

/** The value an instance uses when it sets none. */
export function defaultParamValue(param: TemplateParam): TemplateParamValue {
  const { spec } = param;
  switch (spec.kind) {
    case 'string':
      return { kind: 'text', value: spec.default };
    case 'number':
      return { kind: 'number', value: { kind: 'static', value: spec.default } };
    case 'color':
      return { kind: 'color', value: { kind: 'static', value: spec.default } };
    case 'boolean':
      return { kind: 'boolean', value: spec.default };
    case 'enum':
      return { kind: 'enum', value: spec.default };
    case 'font':
      return { kind: 'font', value: spec.default };
    case 'asset':
      return { kind: 'asset', value: null };
  }
}

/** The value of `param` an instance holds: its own when of the parameter's kind, else the default. */
export function paramValue(param: TemplateParam, values: ParamValues): TemplateParamValue {
  const own = values[param.name];
  return own?.kind === VALUE_KIND[param.spec.kind] ? own : defaultParamValue(param);
}

/** The template of a template instance, if the project holds it. */
export function templateOf(
  item: ItemEntity,
  templates: readonly MotionTemplate[],
): MotionTemplate | null {
  if (item.kind.kind !== 'motionTemplate') return null;
  const { templateId } = item.kind;
  return templates.find((template) => template.id === templateId) ?? null;
}

/** The values of a template instance (none for other items). */
export function valuesOf(item: ItemEntity): ParamValues {
  return item.kind.kind === 'motionTemplate' ? item.kind.params : {};
}

/** A number value at `local` (item-relative time). */
export function numberAt(value: TemplateParamValue, local: Flicks): number | null {
  return value.kind === 'number' ? valueAtTime(value.value, local, interpolateNumber) : null;
}

/** A colour value at `local`. */
export function colorAt(value: TemplateParamValue, local: Flicks): Color | null {
  return value.kind === 'color' ? valueAtTime(value.value, local, interpolateColor) : null;
}

/** `value` holding `number` at `local` (a keyframe there when the value is keyframed). */
export function withNumberAt(
  value: TemplateParamValue,
  local: Flicks,
  number: number,
  tolerance: number,
): TemplateParamValue {
  if (value.kind !== 'number') return value;
  return { kind: 'number', value: withValueAt(value.value, local, number, tolerance) };
}

/** `value` holding `color` at `local` (a keyframe there when the value is keyframed). */
export function withColorAt(
  value: TemplateParamValue,
  local: Flicks,
  color: Color,
  tolerance: number,
): TemplateParamValue {
  if (value.kind !== 'color') return value;
  return { kind: 'color', value: withValueAt(value.value, local, color, tolerance) };
}

/** The keyframe button state of an animatable value at `local` (`none` for the others). */
export function paramKeyframeState(
  value: TemplateParamValue,
  local: Flicks,
  tolerance: number,
): KeyframeState {
  if (value.kind === 'number') return keyframeState(value.value, local, tolerance);
  if (value.kind === 'color') return keyframeState(value.value, local, tolerance);
  return 'none';
}

/** The value with its keyframe at `local` added or removed. */
export function toggledParamKeyframe(
  value: TemplateParamValue,
  local: Flicks,
  tolerance: number,
): TemplateParamValue {
  if (value.kind === 'number') {
    return {
      kind: 'number',
      value: toggledKeyframe(value.value, local, tolerance, interpolateNumber),
    };
  }
  if (value.kind === 'color') {
    return {
      kind: 'color',
      value: toggledKeyframe(value.value, local, tolerance, interpolateColor),
    };
  }
  return value;
}

/** The time of the keyframe before (`-1`) or after (`1`) `local`, if any. */
export function neighbourParamKeyframe(
  value: TemplateParamValue,
  local: Flicks,
  direction: -1 | 1,
  tolerance: number,
): Flicks | null {
  const animatable: Animatable<number> | Animatable<Color> | null =
    value.kind === 'number' || value.kind === 'color' ? value.value : null;
  if (animatable === null) return null;
  return neighbourKeyframe(animatable as Animatable<unknown>, local, direction, tolerance);
}

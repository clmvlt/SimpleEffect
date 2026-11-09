import type {
  AssetId,
  Color,
  Flicks,
  MotionTemplate,
  TemplateParam,
  TemplateParamValue,
} from '@se/protocol';

import { interpolateColor, interpolateNumber, valueAt } from '../animation/interpolate';
import { FLICKS_PER_SECOND } from '../time/flicks';

/** The project's motion templates (prompt 24). */
export type TemplateList = readonly MotionTemplate[];

/** The code a template layer runs: a template's id, version and source. */
export interface TemplateRef {
  readonly id: MotionTemplate['id'];
  readonly version: number;
  readonly source: string;
}

/**
 * A parameter value at the evaluated time, as the template runtime passes it to `render` (asset
 * parameters as their asset id; the runtime turns them into image handles).
 */
export type TemplateValue = string | number | boolean | { readonly assetId: AssetId } | null;

/** The values of every parameter of a template, by name. */
export type TemplateValues = Readonly<Record<string, TemplateValue>>;

/**
 * The value of every declared parameter of `template` at `localTime` (item-relative flicks): the
 * item's value when it has one of the parameter's kind, else the default. Numbers and colours are
 * interpolated like every keyframe (`valueAt`); colours become `#rrggbb[aa]` strings.
 */
export function templateValues(
  template: Pick<MotionTemplate, 'params'>,
  values: Readonly<Record<string, TemplateParamValue | undefined>>,
  localTime: Flicks,
): TemplateValues {
  const resolved: Record<string, TemplateValue> = {};
  for (const param of template.params) {
    resolved[param.name] = paramValue(param, values[param.name], localTime);
  }
  return resolved;
}

function paramValue(
  param: TemplateParam,
  value: TemplateParamValue | undefined,
  localTime: Flicks,
): TemplateValue {
  const spec = param.spec;
  switch (spec.kind) {
    case 'string':
      return value?.kind === 'text' ? value.value : spec.default;
    case 'enum':
      return value?.kind === 'enum' ? value.value : spec.default;
    case 'font':
      return value?.kind === 'font' ? value.value : spec.default;
    case 'boolean':
      return value?.kind === 'boolean' ? value.value : spec.default;
    case 'number':
      return value?.kind === 'number'
        ? valueAt(value.value, localTime, interpolateNumber, spec.default)
        : spec.default;
    case 'color':
      return value?.kind === 'color' ? colorAt(value.value, localTime, spec.default) : spec.default;
    case 'asset':
      return value?.kind === 'asset' && value.value !== null ? { assetId: value.value } : null;
  }
}

function colorAt(
  value: Extract<TemplateParamValue, { kind: 'color' }>['value'],
  localTime: Flicks,
  fallback: Color,
): Color {
  return valueAt(value, localTime, interpolateColor, fallback);
}

/** Seconds of a flicks duration, for template code (which works in seconds). */
export function templateSeconds(time: Flicks): number {
  return time / FLICKS_PER_SECOND;
}

/** The image assets a template layer's values refer to. */
export function templateAssets(values: TemplateValues): AssetId[] {
  const assets: AssetId[] = [];
  for (const value of Object.values(values)) {
    if (typeof value === 'object' && value !== null) assets.push(value.assetId);
  }
  return assets;
}

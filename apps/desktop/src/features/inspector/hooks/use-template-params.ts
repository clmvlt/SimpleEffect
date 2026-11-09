import type {
  Flicks,
  ItemEntity,
  MotionTemplate,
  SequenceEntity,
  TemplateParam,
  TemplateParamValue,
} from '@se/protocol';
import { toast } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { clearPropertyPreview, previewTemplateParam } from '../../../shared/animation';
import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { setPlayheadTime } from '../../timeline';
import { inspectorApi } from '../api';
import { keyframeTolerance, localTime } from '../services/inspector-values';
import {
  neighbourParamKeyframe,
  paramKeyframeState,
  paramValue,
  toggledParamKeyframe,
  valuesOf,
} from '../services/template-params';
import type { KeyframeState } from '../../../shared/animation';

/**
 * A change of one parameter for one item, from its current value, its local time and the
 * keyframe tolerance (half a frame).
 */
export type ParamChange = (
  current: TemplateParamValue,
  local: Flicks,
  tolerance: number,
) => TemplateParamValue;

/** Edits of the parameters of the inspected template instances (one command per gesture). */
export interface TemplateParamsEditor {
  readonly template: MotionTemplate;
  readonly items: readonly ItemEntity[];
  /** The value of `param` of the first instance (its own or the default). */
  readonly value: (param: TemplateParam) => TemplateParamValue;
  /** The first instance's time relative to its start, at the playhead. */
  readonly local: Flicks;
  /** Shows `change` on every instance while a gesture runs (no command). */
  readonly preview: (param: TemplateParam, change: ParamChange) => void;
  /** Applies `change` to every instance: one step. */
  readonly commit: (param: TemplateParam, change: ParamChange) => void;
  /** Resets `param` to the template's default on every instance. */
  readonly reset: (param: TemplateParam) => void;
  readonly keyframe: (param: TemplateParam) => KeyframeState;
  readonly toggleKeyframe: (param: TemplateParam) => void;
  readonly goToKeyframe: (param: TemplateParam, direction: -1 | 1) => (() => void) | undefined;
}

/** The parameters editor of `items` (instances of `template`) of `sequence` at `playhead`. */
export function useTemplateParams(
  projectId: string,
  sequence: SequenceEntity,
  template: MotionTemplate,
  items: readonly ItemEntity[],
  playhead: Flicks,
): TemplateParamsEditor {
  const { t } = useTranslation();
  const client = useCoreClient();
  const api = inspectorApi(client, projectId);
  const tolerance = keyframeTolerance(sequence.format.frameRate);
  const report = (error: unknown) => {
    clearPropertyPreview();
    toast.error(t(errorMessageKey(error)));
  };
  const changed = (param: TemplateParam, change: ParamChange) =>
    items.map((item) => ({
      item,
      value: change(paramValue(param, valuesOf(item)), localTime(item, playhead), tolerance),
    }));
  const send = (param: TemplateParam, values: (TemplateParamValue | null)[]) => {
    const updates = items.map((item, index) => ({
      itemId: item.id,
      patch: {
        kind: 'motionTemplate' as const,
        templateId: null,
        params: { [param.name]: values[index] ?? null },
      },
    }));
    api.updateItems(updates).then(clearPropertyPreview, report);
  };
  const [first] = items;
  const firstLocal = first === undefined ? (0 as Flicks) : localTime(first, playhead);
  const firstValue = (param: TemplateParam) =>
    paramValue(param, first === undefined ? {} : valuesOf(first));
  return {
    template,
    items,
    local: firstLocal,
    value: firstValue,
    preview: (param, change) => {
      for (const { item, value } of changed(param, change)) {
        previewTemplateParam(item.id, param.name, value);
      }
    },
    commit: (param, change) => {
      send(
        param,
        changed(param, change).map(({ value }) => value),
      );
    },
    reset: (param) => {
      send(
        param,
        items.map(() => null),
      );
    },
    keyframe: (param) => paramKeyframeState(firstValue(param), firstLocal, tolerance),
    toggleKeyframe: (param) => {
      send(
        param,
        changed(param, (current, local) => toggledParamKeyframe(current, local, tolerance)).map(
          ({ value }) => value,
        ),
      );
    },
    goToKeyframe: (param, direction) => {
      if (first === undefined) return undefined;
      const time = neighbourParamKeyframe(firstValue(param), firstLocal, direction, tolerance);
      if (time === null) return undefined;
      return () => {
        setPlayheadTime(sequence.id, (first.timelineRange.start + time) as Flicks);
      };
    },
  };
}

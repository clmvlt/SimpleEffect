import type {
  AnimatableProperty,
  Flicks,
  ItemAnimation,
  ItemEntity,
  ItemPatch,
  PropertyAnimation,
  SequenceEntity,
} from '@se/protocol';
import { toast } from '@se/ui';
import { useTranslation } from 'react-i18next';

import {
  animationOf,
  clearPropertyPreview,
  interpolatorOf,
  keyframeState,
  neighbourKeyframe,
  previewAnimations,
  propertyOf,
  toggledKeyframe,
  valueAtTime,
  withValueAt,
  type KeyframeState,
  type PropertyValues,
} from '../../../shared/animation';
import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { setPlayheadTime } from '../../timeline';
import { inspectorApi } from '../api';
import { keyframeTolerance, localTime } from '../services/inspector-values';

/** A patch of one item and the properties set with it (one step). */
export interface ItemEdit {
  readonly patch: ItemPatch;
  readonly animations: readonly PropertyAnimation[];
}

/** Edits of the inspected items at the playhead, each one command (one undo step). */
export interface PropertyEditor {
  readonly items: readonly ItemEntity[];
  /** Value of `property` on the first item at the playhead (`null` when it lacks it). */
  readonly value: <P extends AnimatableProperty>(property: P) => PropertyValues[P] | null;
  /** Values of `property` of every item at the playhead (items lacking it left out). */
  readonly values: <P extends AnimatableProperty>(property: P) => PropertyValues[P][];
  /** The keyframe button state of `property` (first item). */
  readonly keyframe: (property: AnimatableProperty) => KeyframeState;
  /** Shows `value` while a gesture runs (no command). */
  readonly preview: <P extends AnimatableProperty>(property: P, value: PropertyValues[P]) => void;
  /** Sets `property` to `value` at the playhead on every item (keyframed ones get a keyframe). */
  readonly commit: <P extends AnimatableProperty>(property: P, value: PropertyValues[P]) => void;
  /** Adds or removes the keyframe of `property` at the playhead on every item. */
  readonly toggleKeyframe: (property: AnimatableProperty) => void;
  /** Moves the playhead to the previous or next keyframe of `property` (first item). */
  readonly goToKeyframe: (
    property: AnimatableProperty,
    direction: -1 | 1,
  ) => (() => void) | undefined;
  /** Sends one typed patch per item (`patchOf` returns `null` to leave an item out). */
  readonly patch: (patchOf: (item: ItemEntity) => ItemPatch | null) => void;
  /** Sends one patch per item with properties set in the same step (`null` leaves it out). */
  readonly edit: (editOf: (item: ItemEntity) => ItemEdit | null) => void;
}

/** The property editor of `items` of `sequence` at `playhead`, sending to `projectId`. */
export function usePropertyEditor(
  projectId: string,
  sequence: SequenceEntity,
  items: readonly ItemEntity[],
  playhead: Flicks,
): PropertyEditor {
  const { t } = useTranslation();
  const client = useCoreClient();
  const api = inspectorApi(client, projectId);
  const tolerance = keyframeTolerance(sequence.format.frameRate);
  const report = (error: unknown) => {
    clearPropertyPreview();
    toast.error(t(errorMessageKey(error)));
  };
  const changes = <P extends AnimatableProperty>(
    property: P,
    next: (item: ItemEntity, local: Flicks) => ReturnType<typeof propertyOf<P>>,
  ): ItemAnimation[] =>
    items.flatMap((item) => {
      const value = next(item, localTime(item, playhead));
      return value === null ? [] : [{ itemId: item.id, animation: animationOf(property, value) }];
    });
  const setAt = <P extends AnimatableProperty>(property: P, value: PropertyValues[P]) =>
    changes(property, (item, local) => {
      const current = propertyOf(item.kind, property);
      return current === null ? null : withValueAt(current, local, value, tolerance);
    });
  const send = (animations: ItemAnimation[]) => {
    if (animations.length === 0) return;
    api.setItemAnimations(animations).then(clearPropertyPreview, report);
  };
  const values = <P extends AnimatableProperty>(property: P): PropertyValues[P][] =>
    items.flatMap((item) => {
      const current = propertyOf(item.kind, property);
      if (current === null) return [];
      return [valueAtTime(current, localTime(item, playhead), interpolatorOf(property))];
    });
  const [first] = items;
  return {
    items,
    value: (property) => values(property)[0] ?? null,
    values,
    keyframe: (property) => {
      const current = first === undefined ? null : propertyOf(first.kind, property);
      if (first === undefined || current === null) return 'none';
      return keyframeState(current, localTime(first, playhead), tolerance);
    },
    preview: (property, value) => {
      previewAnimations(setAt(property, value));
    },
    commit: (property, value) => {
      send(setAt(property, value));
    },
    toggleKeyframe: (property) => {
      send(
        changes(property, (item, local) => {
          const current = propertyOf(item.kind, property);
          return current === null
            ? null
            : toggledKeyframe(current, local, tolerance, interpolatorOf(property));
        }),
      );
    },
    goToKeyframe: (property, direction) => {
      const current = first === undefined ? null : propertyOf(first.kind, property);
      if (first === undefined || current === null) return undefined;
      const time = neighbourKeyframe(current, localTime(first, playhead), direction, tolerance);
      if (time === null) return undefined;
      return () => {
        setPlayheadTime(sequence.id, (first.timelineRange.start + time) as Flicks);
      };
    },
    patch: (patchOf) => {
      const updates = items.flatMap((item) => {
        const patch = patchOf(item);
        return patch === null ? [] : [{ itemId: item.id, patch }];
      });
      if (updates.length > 0) api.updateItems(updates).catch(report);
    },
    edit: (editOf) => {
      const edits = items.flatMap((item) => {
        const edit = editOf(item);
        return edit === null ? [] : [{ item, edit }];
      });
      if (edits.length === 0) return;
      const updates = edits.map(({ item, edit }) => ({ itemId: item.id, patch: edit.patch }));
      const animations = edits.flatMap(({ item, edit }) =>
        edit.animations.map((animation) => ({ itemId: item.id, animation })),
      );
      api.updateItems(updates, animations).catch(report);
    },
  };
}

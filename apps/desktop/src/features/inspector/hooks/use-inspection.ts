import type { ItemEntity, SequenceEntity } from '@se/protocol';

import { useProjectSelector } from '../../../shared/project-store';
import { useActiveSequence, usePlayheadTime, useSelectedItemIds } from '../../timeline';
import { usePropertyEditor, type PropertyEditor } from './use-property-editor';

/** Item kinds the visual sections inspect. */
export type VisualKind = 'text' | 'shape' | 'image' | 'mediaClip' | 'motionTemplate';

/** The selected visual items of the active sequence and their editor. */
export interface Inspection {
  readonly sequence: SequenceEntity;
  readonly items: readonly ItemEntity[];
  /** The kind shared by every item, or `null` for a mixed selection. */
  readonly kind: VisualKind | null;
  readonly editor: PropertyEditor;
}

function isVisual(kind: ItemEntity['kind']['kind']): kind is VisualKind {
  return (
    kind === 'text' ||
    kind === 'shape' ||
    kind === 'image' ||
    kind === 'mediaClip' ||
    kind === 'motionTemplate'
  );
}

/**
 * The selection seen by the visual sections: its items on video tracks of the active sequence
 * (texts, shapes, images, clips, motion template instances), with their property editor at the
 * playhead; `null` when none.
 */
export function useInspection(): Inspection | null {
  const mirror = useProjectSelector((state) => state.mirror);
  const sequence = useActiveSequence();
  const selected = useSelectedItemIds();
  const playhead = usePlayheadTime(sequence?.id ?? '');
  const items =
    mirror === null
      ? []
      : selected
          .map((id) => mirror.items[id])
          .filter(
            (item): item is ItemEntity =>
              item !== undefined &&
              isVisual(item.kind.kind) &&
              mirror.tracks[item.trackId]?.kind === 'video' &&
              mirror.tracks[item.trackId]?.sequenceId === sequence?.id,
          );
  const editor = usePropertyEditor(
    mirror?.projectId ?? '',
    sequence ?? EMPTY_SEQUENCE,
    items,
    playhead,
  );
  if (mirror === null || sequence === null || items.length === 0) return null;
  const kinds = new Set(items.map((item) => item.kind.kind));
  const [only] = [...kinds];
  const kind = kinds.size === 1 && only !== undefined && isVisual(only) ? only : null;
  return { sequence, items, kind, editor };
}

/** Stands in for the sequence while none is active (the editor is then unused). */
const EMPTY_SEQUENCE: SequenceEntity = {
  id: '',
  name: '',
  format: {
    width: 1920,
    height: 1080,
    frameRate: { numerator: 25, denominator: 1 },
    sampleRate: 48000,
  },
  trackIds: [],
  markerIds: [],
};

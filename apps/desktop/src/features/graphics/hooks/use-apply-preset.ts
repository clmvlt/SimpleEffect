import type { AnimationPhase, AnimationPresetInfo, ItemEntity } from '@se/protocol';
import { toast } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { hasProperty } from '../../../shared/animation';
import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { useProjectSelector } from '../../../shared/project-store';
import { FLICKS_PER_SECOND, clampFlicks } from '../../../shared/time';
import { playheadStore, playheadTimeOf, useSelectedItemIds } from '../../timeline';
import { graphicsApi } from '../api';
import { graphicsPreferencesStore } from '../store/graphics-preferences-store';

/** Applying presets to the selection. */
export interface PresetApplier {
  /** The selected items presets can animate (visual items). */
  readonly items: readonly ItemEntity[];
  /** Whether every selected item has every property `preset` animates. */
  readonly applicable: (preset: AnimationPresetInfo) => boolean;
  /** Applies `preset` as `phase` to the selection (one undo step). */
  readonly apply: (preset: AnimationPresetInfo, phase: AnimationPhase) => void;
}

/**
 * The selection's preset actions. The duration is the preference (or the preset's own); an
 * emphasis on one item starts at the playhead when it is inside the item, else it is centred.
 */
export function useApplyPreset(): PresetApplier {
  const { t } = useTranslation();
  const client = useCoreClient();
  const mirror = useProjectSelector((state) => state.mirror);
  const selected = useSelectedItemIds();
  const items =
    mirror === null
      ? []
      : selected
          .map((id) => mirror.items[id])
          .filter((item): item is ItemEntity => item !== undefined && 'transform' in item.kind)
          .filter((item) => mirror.tracks[item.trackId]?.kind === 'video');
  const applicable = (preset: AnimationPresetInfo) =>
    items.length > 0 &&
    items.every((item) => preset.properties.every((property) => hasProperty(item.kind, property)));
  const apply = (preset: AnimationPresetInfo, phase: AnimationPhase) => {
    if (mirror === null || !applicable(preset)) return;
    const seconds = graphicsPreferencesStore.getState().animationSeconds;
    const duration =
      seconds === null ? preset.defaultDuration : clampFlicks(seconds * FLICKS_PER_SECOND);
    const [only] = items;
    let start = null;
    if (phase === 'emphasis' && items.length === 1 && only !== undefined) {
      const sequenceId = mirror.tracks[only.trackId]?.sequenceId;
      const playhead =
        sequenceId === undefined ? 0 : playheadTimeOf(playheadStore.getState(), sequenceId);
      const offset = playhead - only.timelineRange.start;
      if (offset >= 0 && offset < only.timelineRange.duration) start = offset as typeof duration;
    }
    graphicsApi(client, mirror.projectId)
      .applyAnimationPreset(
        items.map((item) => item.id),
        preset.id,
        phase,
        duration,
        start,
      )
      .catch((error: unknown) => {
        toast.error(t(errorMessageKey(error)));
      });
  };
  return { items, applicable, apply };
}

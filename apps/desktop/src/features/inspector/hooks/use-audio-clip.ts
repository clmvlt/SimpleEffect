import type { Animatable, FadeShape, ItemEntity, MediaClip, MediaClipPatch } from '@se/protocol';
import { toast } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { useProjectSelector } from '../../../shared/project-store';
import { useSelectedItemIds } from '../../timeline';
import { inspectorApi } from '../api';

/** The audio clip the inspector edits and the edits it sends (one undo step each). */
export interface AudioClipEditor {
  readonly item: ItemEntity;
  readonly clip: MediaClip;
  readonly setVolume: (volume: Animatable<number>) => void;
  readonly updateFades: (
    patch: Pick<MediaClipPatch, 'fadeIn' | 'fadeOut' | 'fadeInShape' | 'fadeOutShape'>,
  ) => void;
}

/** Fade shapes offered, in menu order. */
export const FADE_SHAPES: readonly FadeShape[] = ['linear', 'constantPower', 'logarithmic'];

/**
 * The one media clip of an audio track among the selected items (a selected clip with its linked
 * picture counts), or `null` when there is none or several.
 */
export function useAudioClip(): AudioClipEditor | null {
  const { t } = useTranslation();
  const client = useCoreClient();
  const selected = useSelectedItemIds();
  const mirror = useProjectSelector((state) => state.mirror);
  if (mirror === null) return null;
  const audio = selected
    .map((id) => mirror.items[id])
    .filter(
      (item): item is ItemEntity =>
        item !== undefined &&
        item.kind.kind === 'mediaClip' &&
        mirror.tracks[item.trackId]?.kind === 'audio',
    );
  const [item] = audio;
  if (audio.length !== 1 || item === undefined || item.kind.kind !== 'mediaClip') return null;
  const api = inspectorApi(client, mirror.projectId);
  const report = (error: unknown) => {
    toast.error(t(errorMessageKey(error)));
  };
  return {
    item,
    clip: item.kind,
    setVolume: (volume) => {
      api.setItemAnimation(item.id, { property: 'volumeDb', value: volume }).catch(report);
    },
    updateFades: (patch) => {
      api.updateItems([{ itemId: item.id, patch: { kind: 'mediaClip', ...patch } }]).catch(report);
    },
  };
}

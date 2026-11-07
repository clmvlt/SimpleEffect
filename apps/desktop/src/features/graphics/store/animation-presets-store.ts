import type { AnimationPresetInfo, CoreClient } from '@se/protocol';
import { useEffect } from 'react';
import { useStore } from 'zustand';
import { createStore } from 'zustand/vanilla';

import { useCoreClient } from '../../../shared/core';
import { listAnimationPresets } from '../api';

/** The preset library, fetched once from the core. */
export const animationPresetsStore = createStore<{
  readonly presets: readonly AnimationPresetInfo[] | null;
}>()(() => ({ presets: null }));

let loading: Promise<void> | null = null;

/** Fetches the library once (again after a failure). */
export function loadAnimationPresets(client: Pick<CoreClient, 'send'>): Promise<void> {
  if (animationPresetsStore.getState().presets !== null) return Promise.resolve();
  loading ??= listAnimationPresets(client)
    .then((presets) => {
      animationPresetsStore.setState({ presets });
    })
    .finally(() => {
      loading = null;
    });
  return loading;
}

const NONE: readonly AnimationPresetInfo[] = [];

/** The presets (empty until fetched), fetching them on first use. */
export function useAnimationPresets(): readonly AnimationPresetInfo[] {
  const client = useCoreClient();
  const presets = useStore(animationPresetsStore, (state) => state.presets);
  useEffect(() => {
    if (presets === null) loadAnimationPresets(client).catch(() => undefined);
  }, [client, presets]);
  return presets ?? NONE;
}

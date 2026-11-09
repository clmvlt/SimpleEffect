import { toast } from '@se/ui';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { timelineApi, type TimelineApi } from '../api';

/** A timeline call that reports its failure in a toast and resolves with `null` then. */
type Guarded<F> = F extends (...args: infer A) => Promise<infer R>
  ? (...args: A) => Promise<R | null>
  : never;

/** The timeline's protocol calls, failures shown to the user (translated by error code). */
export type TimelineCommands = { readonly [K in keyof TimelineApi]: Guarded<TimelineApi[K]> };

/** The timeline commands of the open project `projectId`. */
export function useTimelineCommands(projectId: string): TimelineCommands {
  const client = useCoreClient();
  const { t } = useTranslation();
  const api = timelineApi(client, projectId);
  const guard =
    <A extends unknown[], R>(call: (...args: A) => Promise<R>) =>
    async (...args: A): Promise<R | null> => {
      try {
        return await call(...args);
      } catch (error) {
        console.warn('[timeline]', error);
        toast.error(t(errorMessageKey(error)));
        return null;
      }
    };
  return {
    addTrack: guard(api.addTrack),
    removeTrack: guard(api.removeTrack),
    updateTrack: guard(api.updateTrack),
    reorderTracks: guard(api.reorderTracks),
    addMarker: guard(api.addMarker),
    updateMarker: guard(api.updateMarker),
    removeMarker: guard(api.removeMarker),
    createSequence: guard(api.createSequence),
    placeMedia: guard(api.placeMedia),
    insertTemplate: guard(api.insertTemplate),
    moveItems: guard(api.moveItems),
    duplicateItems: guard(api.duplicateItems),
    pasteItems: guard(api.pasteItems),
    trimItems: guard(api.trimItems),
    splitItems: guard(api.splitItems),
    deleteItems: guard(api.deleteItems),
    linkItems: guard(api.linkItems),
    unlinkItems: guard(api.unlinkItems),
    resizeGap: guard(api.resizeGap),
    updateItems: guard(api.updateItems),
    setItemAnimation: guard(api.setItemAnimation),
    setItemAnimations: guard(api.setItemAnimations),
    addTransitions: guard(api.addTransitions),
    updateTransition: guard(api.updateTransition),
    removeTransitions: guard(api.removeTransitions),
    getAudioSilences: guard(api.getAudioSilences),
  };
}

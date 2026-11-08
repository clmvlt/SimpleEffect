import type { CutSummary, ProjectId, SequenceId, TimeRange, WordCutPurpose } from '@se/protocol';
import { toast } from '@se/ui';
import { useState } from 'react';
import { useTranslation } from 'react-i18next';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey } from '../../../shared/errors';
import { useProjectMirrorReader } from '../../../shared/project-store';
import { removeSilences, removeWords, type SilenceCutRequest } from '../api';
import { cutSeconds } from '../services/cut-format';
import { restoreTrims, type CutPassage } from '../services/cut-passages';

/** What the transcript panel cuts with. */
export interface SmartCutActions {
  /** Cuts the words whose middle lies in `ranges`; resolves with whether something was cut. */
  readonly cutWords: (ranges: readonly TimeRange[], purpose: WordCutPurpose) => Promise<boolean>;
  /** Cuts silences; resolves with whether something was cut. */
  readonly cutSilences: (
    request: Omit<SilenceCutRequest, 'projectId' | 'sequenceId'>,
  ) => Promise<boolean>;
  /** Puts a struck passage back with a ripple trim of the clip before it. */
  readonly restore: (passage: CutPassage) => void;
  /** Whether a cut is being made. */
  readonly busy: boolean;
}

/**
 * The smart cuts of the transcript panel on `sequenceId`: each one undoable transaction, then a
 * toast saying how many cuts and how much time was gained, with « Annuler ».
 */
export function useSmartCuts(
  projectId: ProjectId | null,
  sequenceId: SequenceId | null,
): SmartCutActions {
  const { t } = useTranslation();
  const client = useCoreClient();
  const readMirror = useProjectMirrorReader();
  const [busy, setBusy] = useState(false);
  const undo = () => {
    if (projectId === null) return;
    client.send({ kind: 'undo', projectId }).catch((error: unknown) => {
      toast.error(t(errorMessageKey(error)));
    });
  };
  const report = (summary: CutSummary): boolean => {
    if (summary.cuts.length === 0) {
      toast.info(t('editor.transcript.cuts.nothing'));
      return false;
    }
    toast.success(
      t('editor.transcript.cuts.done', {
        count: summary.cuts.length,
        seconds: cutSeconds(summary.removedDuration),
      }),
      { action: { label: t('editor.transcript.cuts.undo'), onClick: undo } },
    );
    return true;
  };
  const run = async (cut: () => Promise<CutSummary>): Promise<boolean> => {
    setBusy(true);
    try {
      return report(await cut());
    } catch (error: unknown) {
      toast.error(t(errorMessageKey(error)));
      return false;
    } finally {
      setBusy(false);
    }
  };
  return {
    busy,
    cutWords: (ranges, purpose) => {
      if (projectId === null || sequenceId === null || ranges.length === 0) {
        return Promise.resolve(false);
      }
      return run(() =>
        removeWords(client, { projectId, sequenceId, ranges, purpose, allTracks: false }),
      );
    },
    cutSilences: (request) => {
      if (projectId === null || sequenceId === null) return Promise.resolve(false);
      return run(() => removeSilences(client, { ...request, projectId, sequenceId }));
    },
    restore: (passage) => {
      const mirror = readMirror();
      const trims = mirror === null ? null : restoreTrims(passage, mirror);
      if (projectId === null || trims === null) {
        toast.error(t('editor.transcript.passages.cannotRestore'));
        return;
      }
      client.send({ kind: 'trimItems', projectId, trims, ripple: true }).catch((error: unknown) => {
        toast.error(t('editor.transcript.passages.cannotRestore'), {
          description: t(errorMessageKey(error)),
        });
      });
    },
  };
}

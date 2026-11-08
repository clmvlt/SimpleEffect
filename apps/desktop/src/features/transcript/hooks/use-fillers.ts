import type { AssetId, FillerOccurrence, ProjectId, SequenceId } from '@se/protocol';
import { useEffect, useState } from 'react';

import { useCoreClient } from '../../../shared/core';
import { errorMessageKey, type ErrorMessageKey } from '../../../shared/errors';
import { useProjectSelector } from '../../../shared/project-store';
import { findFillers } from '../api';

/** The filler words of a sequence, found by the core. */
export interface FillerSearch {
  /** In sequence order; `null` while the first search runs. */
  readonly occurrences: readonly FillerOccurrence[] | null;
  /** Media of the sequence not transcribed yet. */
  readonly untranscribed: readonly AssetId[];
  /** The i18n key of the last failure. */
  readonly errorKey: ErrorMessageKey | null;
}

/**
 * The fillers of `sequenceId`, searched again whenever the project changes (a cut, a new
 * transcript) or the lists change (`listsKey`).
 */
export function useFillers(
  projectId: ProjectId,
  sequenceId: SequenceId,
  listsKey: string,
): FillerSearch {
  const client = useCoreClient();
  const revision = useProjectSelector((state) => state.mirror?.revision ?? 0);
  const [search, setSearch] = useState<FillerSearch>({
    occurrences: null,
    untranscribed: [],
    errorKey: null,
  });
  useEffect(() => {
    let current = true;
    findFillers(client, projectId, sequenceId, null).then(
      ({ occurrences, untranscribedAssets }) => {
        if (current) setSearch({ occurrences, untranscribed: untranscribedAssets, errorKey: null });
      },
      (error: unknown) => {
        if (current) {
          setSearch({ occurrences: [], untranscribed: [], errorKey: errorMessageKey(error) });
        }
      },
    );
    return () => {
      current = false;
    };
  }, [client, projectId, sequenceId, revision, listsKey]);
  return search;
}

import type { AssetId, SequenceId } from '@se/protocol';
import { useEffect, useState } from 'react';

import { useCoreClient, useCoreEvent } from '../../../shared/core';
import { useProjectSelector } from '../../../shared/project-store';
import { getTranscriptionStatus } from '../api';
import {
  speechReadiness,
  untranscribedAssets,
  type SpeechReadiness,
} from '../services/composer-notices';

/** Media of a sequence the agent could not read, and why it could not transcribe them. */
export interface SpeechWarning {
  /** Assets of the sequence's sound clips without a transcript. */
  readonly assetIds: readonly AssetId[];
  /** `noModel` or `unavailable`: agents transcribe on their own once it is `ready`. */
  readonly readiness: Exclude<SpeechReadiness, 'ready'>;
}

/**
 * Warns when the sequence plays untranscribed media and transcription cannot run on this machine:
 * the agent would then be unable to read what is said (agents transcribe by themselves when a
 * model is installed, ADR-0034). The status is fetched while some media lack a transcript and
 * followed through `transcriptionStatusChanged`.
 */
export function useSpeechWarning(sequenceId: SequenceId | null): SpeechWarning | null {
  const client = useCoreClient();
  const mirror = useProjectSelector((state) => state.mirror);
  const [readiness, setReadiness] = useState<SpeechReadiness | null>(null);
  const assetIds =
    mirror === null || sequenceId === null ? [] : untranscribedAssets(mirror, sequenceId);
  const needed = assetIds.length > 0;
  useEffect(() => {
    if (!needed) return;
    let current = true;
    getTranscriptionStatus(client).then(
      (status) => {
        if (current) setReadiness(speechReadiness(status));
      },
      () => {
        // Unknown status: no warning rather than a wrong one.
      },
    );
    return () => {
      current = false;
    };
  }, [client, needed]);
  useCoreEvent('transcriptionStatusChanged', (event) => {
    setReadiness(speechReadiness(event.status));
  });
  if (!needed || readiness === null || readiness === 'ready') return null;
  return { assetIds, readiness };
}

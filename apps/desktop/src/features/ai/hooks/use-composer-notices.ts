import { useProjectSelector } from '../../../shared/project-store';
import { smallSelection, type SpeechReadiness } from '../services/composer-notices';
import { fileName } from '../services/item-label';
import { shortRange } from '../services/short-time';
import type { ComposerScope } from './use-composer-scope';
import type { ResumeOption } from './use-composer-submit';
import { useSpeechWarning } from './use-speech-warning';

/** What the composer points out above its bottom bar. */
export interface ComposerNoticeData {
  /** The scope comes from a selection covering little of the sequence. */
  readonly smallSelection: { readonly range: string; readonly tracks: string } | null;
  /** Untranscribed media the agent could not read, since transcription cannot run. */
  readonly speech: {
    readonly readiness: Exclude<SpeechReadiness, 'ready'>;
    readonly count: number;
    /** File name of the first one. */
    readonly name: string;
  } | null;
  /** The session the prompt may resume. */
  readonly resume: ResumeOption | null;
}

/** The composer's notices, `null` when there is nothing to say. */
export function useComposerNotices(
  scope: ComposerScope | null,
  resume: ResumeOption | null,
): ComposerNoticeData | null {
  const assets = useProjectSelector((state) => state.mirror?.assets);
  const speechWarning = useSpeechWarning(scope?.sequence.id ?? null);
  if (scope === null) return null;
  const range = smallSelection(scope.derived, scope.sequenceEnd);
  const scopeTracks = scope.derived.scope.tracks;
  const small =
    range === null
      ? null
      : {
          range: shortRange(range),
          tracks: scope.tracks
            .filter((track) => scopeTracks === null || scopeTracks.includes(track.id))
            .map((track) => track.name)
            .join(', '),
        };
  const firstId = speechWarning?.assetIds[0];
  const first = firstId === undefined ? undefined : assets?.[firstId];
  const speech =
    speechWarning === null
      ? null
      : {
          readiness: speechWarning.readiness,
          count: speechWarning.assetIds.length,
          name: first === undefined ? '' : fileName(first.originalPath),
        };
  if (small === null && speech === null && resume === null) return null;
  return { smallSelection: small, speech, resume };
}

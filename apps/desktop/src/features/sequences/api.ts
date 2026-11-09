import type {
  AutoReframeReport,
  CoreClient,
  Format,
  FormatAdaptation,
  SequenceId,
  TrackingSettings,
} from '@se/protocol';

/** A sequence copy: its id, and what automatic reframing did when it was asked (prompt 26). */
export interface SequenceCopy {
  readonly sequenceId: SequenceId;
  readonly reframe: AutoReframeReport | null;
}

/**
 * Protocol calls on sequences (prompt 16). Each is one transaction in the core (one undo step);
 * the change comes back as a `projectPatched` event that updates the mirror before the call
 * resolves.
 */
export interface SequencesApi {
  readonly renameSequence: (sequenceId: SequenceId, name: string) => Promise<void>;
  readonly setSequenceFormat: (
    sequenceId: SequenceId,
    format: Format,
    adaptation: FormatAdaptation,
  ) => Promise<void>;
  /**
   * A copy after the sequence: `format: null` keeps its format, else a version in `format`;
   * with `autoReframe`, its clips follow their subject.
   */
  readonly duplicateSequence: (
    sequenceId: SequenceId,
    name: string,
    format: Format | null,
    adaptation: FormatAdaptation,
    autoReframe?: TrackingSettings,
  ) => Promise<SequenceCopy>;
  readonly removeSequence: (sequenceId: SequenceId) => Promise<void>;
}

/** The sequence calls of project `projectId` through `client`. */
export function sequencesApi(client: Pick<CoreClient, 'send'>, projectId: string): SequencesApi {
  return {
    renameSequence: async (sequenceId, name) => {
      await client.send({ kind: 'renameSequence', projectId, sequenceId, name });
    },
    setSequenceFormat: async (sequenceId, format, adaptation) => {
      await client.send({ kind: 'setSequenceFormat', projectId, sequenceId, format, adaptation });
    },
    duplicateSequence: async (sequenceId, name, format, adaptation, autoReframe) => {
      const result = await client.send({
        kind: 'duplicateSequence',
        projectId,
        sequenceId,
        name,
        format,
        adaptation,
        ...(autoReframe === undefined ? {} : { autoReframe }),
      });
      return { sequenceId: result.sequenceId, reframe: result.reframe ?? null };
    },
    removeSequence: async (sequenceId) => {
      await client.send({ kind: 'removeSequence', projectId, sequenceId });
    },
  };
}

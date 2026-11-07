import type { CoreClient, Format, FormatAdaptation, SequenceId } from '@se/protocol';

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
  /** A copy after the sequence: `format: null` keeps its format, else a version in `format`. */
  readonly duplicateSequence: (
    sequenceId: SequenceId,
    name: string,
    format: Format | null,
    adaptation: FormatAdaptation,
  ) => Promise<SequenceId>;
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
    duplicateSequence: async (sequenceId, name, format, adaptation) =>
      (
        await client.send({
          kind: 'duplicateSequence',
          projectId,
          sequenceId,
          name,
          format,
          adaptation,
        })
      ).sequenceId,
    removeSequence: async (sequenceId) => {
      await client.send({ kind: 'removeSequence', projectId, sequenceId });
    },
  };
}

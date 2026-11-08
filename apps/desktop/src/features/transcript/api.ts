import type {
  AssetId,
  CoreClient,
  CutSummary,
  FillerOccurrence,
  JobId,
  ProjectId,
  SequenceId,
  SilenceCutParams,
  SmartCutSettings,
  TimeRange,
  TrackId,
  Transcript,
  TranscriptionSettings,
  WordCutPurpose,
} from '@se/protocol';

/** What the transcript feature needs from the core client. */
export type TranscriptClient = Pick<CoreClient, 'send'>;

/** The transcript of an asset's current content, or `null` while it is not transcribed. */
export async function getTranscript(
  client: TranscriptClient,
  projectId: ProjectId,
  assetId: AssetId,
): Promise<Transcript | null> {
  return (await client.send({ kind: 'getTranscript', projectId, assetId })).transcript;
}

/** Queues the transcription of assets (one job each; already running ones are skipped). */
export async function transcribeAssets(
  client: TranscriptClient,
  projectId: ProjectId,
  assetIds: readonly AssetId[],
): Promise<JobId[]> {
  const result = await client.send({
    kind: 'transcribeAssets',
    projectId,
    assetIds: [...assetIds],
  });
  return result.jobIds;
}

/** Sets the project's transcription language and vocabulary (one undo step). */
export async function setTranscriptionSettings(
  client: TranscriptClient,
  projectId: ProjectId,
  settings: TranscriptionSettings,
): Promise<void> {
  await client.send({ kind: 'setTranscriptionSettings', projectId, settings });
}

/** `result` without its `kind` tag (results whose payload sits next to the tag). */
function payloadOf<T extends { readonly kind: string }>(result: T): Omit<T, 'kind'> {
  return Object.fromEntries(Object.entries(result).filter(([key]) => key !== 'kind')) as Omit<
    T,
    'kind'
  >;
}

/** The smart-cut settings (filler lists, silence defaults). */
export async function getSmartCutSettings(client: TranscriptClient): Promise<SmartCutSettings> {
  return payloadOf(await client.send({ kind: 'getSmartCutSettings' }));
}

/** Saves the smart-cut settings; resolves with them as stored. */
export async function updateSmartCutSettings(
  client: TranscriptClient,
  settings: SmartCutSettings,
): Promise<SmartCutSettings> {
  return payloadOf(
    await client.send({
      kind: 'updateSmartCutSettings',
      ...settings,
    }),
  );
}

/** Cuts the words whose middle lies in `ranges` (one undo step). */
export async function removeWords(
  client: TranscriptClient,
  request: {
    readonly projectId: ProjectId;
    readonly sequenceId: SequenceId;
    readonly ranges: readonly TimeRange[];
    readonly purpose: WordCutPurpose;
    readonly allTracks: boolean;
  },
): Promise<CutSummary> {
  return payloadOf(
    await client.send({
      kind: 'removeWords',
      ...request,
      ranges: [...request.ranges],
    }),
  );
}

/** The filler words of a range of a sequence (`null`: the whole sequence). */
export async function findFillers(
  client: TranscriptClient,
  projectId: ProjectId,
  sequenceId: SequenceId,
  range: TimeRange | null,
): Promise<{ occurrences: FillerOccurrence[]; untranscribedAssets: AssetId[] }> {
  const { occurrences, untranscribedAssets } = await client.send({
    kind: 'findFillers',
    projectId,
    sequenceId,
    range,
  });
  return { occurrences, untranscribedAssets };
}

/** A silence removal as the panel describes it. */
export interface SilenceCutRequest {
  readonly projectId: ProjectId;
  readonly sequenceId: SequenceId;
  readonly range: TimeRange | null;
  readonly params: SilenceCutParams;
  readonly trackIds: readonly TrackId[] | null;
  readonly allTracks: boolean;
}

function silenceCommand(request: SilenceCutRequest) {
  return {
    ...request,
    trackIds: request.trackIds === null ? null : [...request.trackIds],
  };
}

/** The cuts a silence removal would make (nothing changes). */
export async function previewSilenceCuts(
  client: TranscriptClient,
  request: SilenceCutRequest,
): Promise<CutSummary> {
  return payloadOf(
    await client.send({
      kind: 'previewSilenceCuts',
      ...silenceCommand(request),
    }),
  );
}

/** Cuts the silences (one undo step). */
export async function removeSilences(
  client: TranscriptClient,
  request: SilenceCutRequest,
): Promise<CutSummary> {
  return payloadOf(
    await client.send({
      kind: 'removeSilences',
      ...silenceCommand(request),
    }),
  );
}

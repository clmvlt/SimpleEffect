import type {
  AssetId,
  CoreClient,
  JobId,
  ProjectId,
  Transcript,
  TranscriptionSettings,
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

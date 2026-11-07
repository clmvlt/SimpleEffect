import type {
  CoreClient,
  DecodabilityReport,
  ImportReport,
  JobSnapshot,
  RelinkAssetRequest,
  RelinkOutcome,
} from '@se/protocol';

/** What the media bin needs from the core client. */
export type MediaClient = Pick<CoreClient, 'send'>;

/** Imports files (absolute paths) into the open project. */
export async function importMedia(
  client: MediaClient,
  projectId: string,
  paths: readonly string[],
): Promise<ImportReport> {
  const result = await client.send({ kind: 'importMedia', projectId, paths: [...paths] });
  return { imported: result.imported, duplicates: result.duplicates, failures: result.failures };
}

/** Points an asset at another file; `contentDiffers` asks for confirmation (send again with `force`). */
export async function relinkAsset(
  client: MediaClient,
  request: RelinkAssetRequest,
): Promise<RelinkOutcome> {
  return (await client.send({ kind: 'relinkAsset', ...request })).outcome;
}

/** Removes an asset no item uses (undoable). */
export async function removeAsset(
  client: MediaClient,
  projectId: string,
  assetId: string,
): Promise<void> {
  await client.send({ kind: 'removeAsset', projectId, assetId });
}

/** The jobs of the open project (queued, running, finished in this session). */
export async function listJobs(client: MediaClient, projectId: string): Promise<JobSnapshot[]> {
  return (await client.send({ kind: 'listJobs', projectId })).jobs;
}

/** Tells the core whether WebCodecs decodes an asset's streams on this machine. */
export async function reportDecodability(
  client: MediaClient,
  report: DecodabilityReport,
): Promise<void> {
  await client.send({ kind: 'reportDecodability', ...report });
}

import type {
  AiSettings,
  CoreClient,
  DownloadTarget,
  MediaSettings,
  MediaToolsStatus,
  ProjectCacheInfo,
  TranscriptionPreferences,
  TranscriptionStatus,
} from '@se/protocol';

/** What the settings feature needs from the core client. */
export type SettingsClient = Pick<CoreClient, 'send'>;

/** The media settings of the app. */
export async function getMediaSettings(client: SettingsClient): Promise<MediaSettings> {
  return (await client.send({ kind: 'getMediaSettings' })).settings;
}

/** Saves the media settings; resolves with them as saved. */
export async function updateMediaSettings(
  client: SettingsClient,
  settings: MediaSettings,
): Promise<MediaSettings> {
  return (await client.send({ kind: 'updateMediaSettings', ...settings })).settings;
}

/** The ffmpeg tools in use and the hardware encoders that work. */
export async function getMediaToolsStatus(client: SettingsClient): Promise<MediaToolsStatus> {
  return (await client.send({ kind: 'getMediaToolsStatus' })).status;
}

/** The cache folder of an open project and its size. */
export async function getProjectCache(
  client: SettingsClient,
  projectId: string,
): Promise<ProjectCacheInfo> {
  return (await client.send({ kind: 'getProjectCache', projectId })).cache;
}

/** Deletes the project's cache (derived files are generated again as needed). */
export async function clearProjectCache(
  client: SettingsClient,
  projectId: string,
): Promise<ProjectCacheInfo> {
  return (await client.send({ kind: 'clearProjectCache', projectId })).cache;
}

/** The AI settings of the app. */
export async function getAiSettings(client: SettingsClient): Promise<AiSettings> {
  return (await client.send({ kind: 'getAiSettings' })).settings;
}

/** Saves the AI settings; resolves with them as saved. */
export async function updateAiSettings(
  client: SettingsClient,
  settings: AiSettings,
): Promise<AiSettings> {
  return (await client.send({ kind: 'updateAiSettings', ...settings })).settings;
}

/** What « Réglages → Transcription » shows: whisper.cpp, the GPU, models and preferences. */
export async function getTranscriptionStatus(client: SettingsClient): Promise<TranscriptionStatus> {
  return (await client.send({ kind: 'getTranscriptionStatus' })).status;
}

/** Saves the transcription preferences; resolves with the status holding them as saved. */
export async function updateTranscriptionPreferences(
  client: SettingsClient,
  preferences: TranscriptionPreferences,
): Promise<TranscriptionStatus> {
  return (await client.send({ kind: 'updateTranscriptionPreferences', ...preferences })).status;
}

/**
 * Starts downloading a model, the voice model or the GPU pack (only after the user consented);
 * progress arrives as `transcriptionStatusChanged` events.
 */
export async function startTranscriptionDownload(
  client: SettingsClient,
  target: DownloadTarget,
): Promise<void> {
  await client.send({ kind: 'startTranscriptionDownload', target });
}

/** Stops a running download (a `transcriptionStatusChanged` event follows). */
export async function cancelTranscriptionDownload(
  client: SettingsClient,
  target: DownloadTarget,
): Promise<void> {
  await client.send({ kind: 'cancelTranscriptionDownload', target });
}

/** Copies a ggml `.bin` file into the models folder; resolves with its id and the new status. */
export async function importTranscriptionModel(
  client: SettingsClient,
  path: string,
): Promise<{ readonly modelId: string; readonly status: TranscriptionStatus }> {
  const { modelId, status } = await client.send({ kind: 'importTranscriptionModel', path });
  return { modelId, status };
}

/** Deletes an installed model, the voice model or the GPU pack; resolves with the new status. */
export async function removeTranscriptionFile(
  client: SettingsClient,
  target: DownloadTarget,
): Promise<TranscriptionStatus> {
  return (await client.send({ kind: 'removeTranscriptionFile', target })).status;
}

import type { CoreClient, MediaSettings, MediaToolsStatus, ProjectCacheInfo } from '@se/protocol';

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

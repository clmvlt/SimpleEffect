import type { CoreClient, MediaToolsStatus, NewProjectDefaults } from '@se/protocol';

/** What the onboarding feature needs from the core client. */
export type OnboardingClient = Pick<CoreClient, 'send'>;

/** The ffmpeg tools found and the hardware encoders that work. */
export async function getMediaToolsStatus(client: OnboardingClient): Promise<MediaToolsStatus> {
  return (await client.send({ kind: 'getMediaToolsStatus' })).status;
}

/** The folder new projects are proposed in (`parentDir`). */
export async function getProjectsFolder(client: OnboardingClient): Promise<string> {
  return (await client.send({ kind: 'getNewProjectDefaults' })).parentDir;
}

/** Makes `path` (absolute) the folder new projects are proposed in; resolves with the defaults. */
export async function setProjectsFolder(
  client: OnboardingClient,
  path: string,
): Promise<NewProjectDefaults> {
  return (await client.send({ kind: 'setProjectsFolder', path })).defaults;
}

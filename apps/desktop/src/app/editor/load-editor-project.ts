import { CoreRequestError, type CoreClient, type ProjectSession } from '@se/protocol';

/**
 * The session of the project the editor's URL names. Usually the core has it open already (the
 * home screen just opened it); after an app reload or a core restart it may not, and the project
 * is then reopened from its folder, found in the recent list.
 */
export async function loadEditorProject(
  client: Pick<CoreClient, 'send'>,
  projectId: string,
): Promise<ProjectSession> {
  try {
    return await client.send({ kind: 'getProjectSnapshot', projectId });
  } catch (error) {
    if (!(error instanceof CoreRequestError && error.code === 'PROJECT_NOT_OPEN')) {
      throw error;
    }
    const { projects } = await client.send({ kind: 'listRecentProjects' });
    const recent = projects.find((project) => project.id === projectId);
    if (recent === undefined) {
      throw error;
    }
    return client.send({ kind: 'openProject', path: recent.path });
  }
}

import type {
  CoreClient,
  CreateProjectRequest,
  NewProjectDefaults,
  ProjectSession,
  RecentProject,
} from '@se/protocol';

/** What the projects feature needs from the core client. */
export type ProjectsClient = Pick<CoreClient, 'send'>;

/** The recent projects, most recently opened first. */
export async function listRecentProjects(client: ProjectsClient): Promise<RecentProject[]> {
  return (await client.send({ kind: 'listRecentProjects' })).projects;
}

/** Choices offered by the « new project » dialog. */
export function getNewProjectDefaults(client: ProjectsClient): Promise<NewProjectDefaults> {
  return client.send({ kind: 'getNewProjectDefaults' });
}

/** Creates and opens a project. */
export function createProject(
  client: ProjectsClient,
  request: CreateProjectRequest,
): Promise<ProjectSession> {
  return client.send({ kind: 'createProject', ...request });
}

/** Opens the project folder at `path`. */
export function openProject(client: ProjectsClient, path: string): Promise<ProjectSession> {
  return client.send({ kind: 'openProject', path });
}

/** Renames the project at `path`. */
export function renameProject(
  client: ProjectsClient,
  path: string,
  name: string,
): Promise<RecentProject> {
  return client.send({ kind: 'renameProject', path, name });
}

/** Copies the project at `path` as `name`. */
export function duplicateProject(
  client: ProjectsClient,
  path: string,
  name: string,
): Promise<RecentProject> {
  return client.send({ kind: 'duplicateProject', path, name });
}

/** Moves the project at `path` to the OS trash. */
export async function trashProject(client: ProjectsClient, path: string): Promise<void> {
  await client.send({ kind: 'trashProject', path });
}

/** Route pattern of the editor (React Router syntax). */
export const PROJECT_ROUTE_PATTERN = 'project/:projectId';

/** Path of the editor showing project `projectId`. */
export function projectPath(projectId: string): string {
  return `/project/${encodeURIComponent(projectId)}`;
}

/** Path of the home screen. */
export const HOME_PATH = '/';

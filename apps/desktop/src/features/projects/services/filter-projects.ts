import type { RecentProject } from '@se/protocol';

import { searchable } from '../../../shared/text';

/** The projects whose name or folder contains `query` (accents and case ignored). */
export function filterProjects(
  projects: readonly RecentProject[],
  query: string,
): readonly RecentProject[] {
  const needle = searchable(query.trim());
  if (needle === '') {
    return projects;
  }
  return projects.filter(
    (project) =>
      searchable(project.name).includes(needle) || searchable(project.path).includes(needle),
  );
}

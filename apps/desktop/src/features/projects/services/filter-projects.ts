import type { RecentProject } from '@se/protocol';

/** `text` lowercased without accents, so « eté » finds « Été ». */
function searchable(text: string): string {
  return text
    .normalize('NFD')
    .replace(/\p{Diacritic}/gu, '')
    .toLowerCase();
}

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

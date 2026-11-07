import type { RecentProject } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { filterProjects } from './filter-projects';

function project(name: string, path: string): RecentProject {
  return {
    id: name,
    name,
    path,
    format: {
      width: 1920,
      height: 1080,
      frameRate: { numerator: 25, denominator: 1 },
      sampleRate: 48_000,
    },
    lastOpenedAt: 0,
    available: true,
  };
}

describe('filterProjects', () => {
  const projects = [
    project('Été à Paris', '/films/ete.seproj'),
    project('Short', '/clients/acme.seproj'),
  ];

  it('matches names and folders, ignoring accents and case', () => {
    expect(filterProjects(projects, 'ETE').map((p) => p.name)).toEqual(['Été à Paris']);
    expect(filterProjects(projects, 'acme').map((p) => p.name)).toEqual(['Short']);
  });

  it('returns everything for an empty query', () => {
    expect(filterProjects(projects, '  ')).toBe(projects);
  });
});

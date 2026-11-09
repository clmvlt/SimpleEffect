import { presetStyle } from '@se/engine/caption-styles';
import type { LibraryTemplate } from '@se/protocol';

import { badgeTemplate } from '../../../shared/project-store/testing/template-fixture';

/** The library entry `entry-1`: the fixture template `badge` saved as « Bandeau ». */
export function libraryEntry({ version }: { readonly version: number }): LibraryTemplate {
  const badge = badgeTemplate();
  return {
    id: 'entry-1',
    version,
    name: 'Bandeau',
    description: 'Nom et fonction',
    tags: ['interview'],
    updatedAt: 0,
    content: {
      kind: 'motionTemplate',
      category: 'lowerThird',
      source: badge.source,
      params: badge.params,
      defaultDuration: badge.defaultDuration,
    },
  };
}

/** The library entry `style-1`: the `box` caption preset saved as « Sobre ». */
export function libraryStyle(): LibraryTemplate {
  return {
    id: 'style-1',
    version: 1,
    name: 'Sobre',
    description: '',
    tags: [],
    updatedAt: 0,
    content: { kind: 'captionStyle', style: presetStyle('box', 'style-in-library') },
  };
}

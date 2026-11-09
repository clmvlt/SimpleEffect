import { presetStyle } from '@se/engine/caption-styles';
import type { BrandKit, LibraryBrandKit, ProjectSession } from '@se/protocol';

import { documentColor } from '../../../shared/project-store/testing/graphics-fixture';
import { session } from '../../../shared/project-store/testing/session-fixture';

/** The library kit `kit-1` « Acme », at `version`, with one logo. */
export function libraryKit(version = 2, overrides: Partial<LibraryBrandKit> = {}): LibraryBrandKit {
  return {
    id: 'kit-1',
    version,
    updatedAt: 0,
    name: 'Acme',
    fonts: { heading: 'Montserrat', body: 'Inter' },
    colors: {
      primary: documentColor('2f6bff'),
      secondary: documentColor('1a1a2e'),
      accent: documentColor('ffcc00'),
      text: documentColor('ffffff'),
      background: documentColor('111111'),
    },
    logos: [{ name: 'Logo', file: 'logos/a.png' }],
    textStyle: {
      fontFamily: 'Montserrat',
      fontWeight: 700,
      italic: false,
      fontSize: { kind: 'static', value: 96 },
      color: { kind: 'static', value: documentColor('ffffff') },
      stroke: {
        enabled: false,
        color: { kind: 'static', value: documentColor('000000') },
        width: { kind: 'static', value: 4 },
      },
      shadow: {
        enabled: false,
        color: { kind: 'static', value: documentColor('000000') },
        blur: { kind: 'static', value: 8 },
        offset: { kind: 'static', value: { x: 0, y: 4 } },
      },
      background: {
        enabled: false,
        color: { kind: 'static', value: documentColor('000000') },
        padding: { x: 24, y: 12 },
        cornerRadius: 8,
      },
      letterSpacing: { kind: 'static', value: 0 },
      lineHeight: 1.2,
      align: 'center',
      maxWidth: null,
      textCase: 'normal',
    },
    captionStyle: presetStyle('minimal', 'kit-caption'),
    introTemplates: [],
    outroTemplates: [],
    ...overrides,
  };
}

/** The project copy of `kit` (made at `version`). */
export function projectCopy(kit: LibraryBrandKit, version: number): BrandKit {
  return {
    source: { kitId: kit.id, version },
    name: kit.name,
    fonts: kit.fonts,
    colors: kit.colors,
    logos: kit.logos.map((logo) => ({ name: logo.name, assetId: 'asset-logo' })),
    textStyle: kit.textStyle,
    captionStyle: kit.captionStyle,
    introTemplates: kit.introTemplates,
    outroTemplates: kit.outroTemplates,
  };
}

/** A session whose project uses `kit` (or none). */
export function kitSession(kit: BrandKit | null): ProjectSession {
  const base = session();
  return {
    ...base,
    snapshot: { ...base.snapshot, project: { ...base.snapshot.project, brandKit: kit } },
  };
}

import { describe, expect, it } from 'vitest';

import { documentColor } from '../../../shared/project-store/testing/graphics-fixture';
import { libraryKit, projectCopy } from '../testing/kit-fixture';
import {
  isDirty,
  withAddedLogos,
  withCaptionPreset,
  withColor,
  withFavorite,
  withFont,
  withLogoName,
  withoutFavorite,
  withoutLogo,
  withTextSize,
  withTextStyle,
} from './kit-draft';
import { projectKitStatus } from './kit-status';

describe('the project kit’s status', () => {
  it('tells none, linked (with a newer version), orphan and not listed yet', () => {
    const kit = libraryKit(3);
    expect(projectKitStatus(null, [kit])).toEqual({ kind: 'none' });
    expect(projectKitStatus(projectCopy(kit, 2), null)).toEqual({ kind: 'unknown' });
    expect(projectKitStatus(projectCopy(kit, 2), [kit])).toEqual({
      kind: 'linked',
      kit,
      newer: kit,
    });
    expect(projectKitStatus(projectCopy(kit, 3), [kit])).toEqual({
      kind: 'linked',
      kit,
      newer: null,
    });
    expect(projectKitStatus(projectCopy(kit, 3), [])).toEqual({ kind: 'orphan' });
  });
});

describe('kit drafts', () => {
  it('edits colours, sizes, logos, favourites and caption looks', () => {
    const kit = libraryKit();
    expect(withColor(kit, 'accent', documentColor('00ff88')).colors.accent).toBe(
      documentColor('00ff88'),
    );
    expect(withTextSize(kit, 120).textStyle.fontSize).toEqual({ kind: 'static', value: 120 });
    expect(withLogoName(kit, 'logos/a.png', 'Blanc').logos[0]?.name).toBe('Blanc');
    expect(withoutLogo(kit, 'logos/a.png').logos).toEqual([]);
    const entry = { id: 'entry-1', name: 'Intro' } as never;
    const favorite = withFavorite(kit, 'introTemplates', entry);
    expect(favorite.introTemplates).toEqual([{ id: 'entry-1', name: 'Intro' }]);
    expect(withFavorite(favorite, 'introTemplates', entry)).toBe(favorite);
    expect(withoutFavorite(favorite, 'introTemplates', 'entry-1').introTemplates).toEqual([]);
    const karaoke = withCaptionPreset(kit, 'karaoke');
    expect(karaoke.captionStyle.preset).toBe('karaoke');
    expect(karaoke.captionStyle.fontFamily).toBe('Inter');
    expect(karaoke.captionStyle.id).toBe(kit.captionStyle.id);
  });

  it('changes fonts, the looks using the old font following', () => {
    const kit = withTextStyle(libraryKit(), { fontFamily: 'Montserrat' });
    const heading = withFont(kit, 'heading', 'Playfair Display');
    expect(heading.fonts).toEqual({ heading: 'Playfair Display', body: 'Inter' });
    expect(heading.textStyle.fontFamily).toBe('Playfair Display');
    expect(heading.captionStyle.fontFamily).toBe('Inter');
    const body = withFont(heading, 'body', 'Oswald');
    expect(body.captionStyle.fontFamily).toBe('Oswald');
    expect(body.textStyle.fontFamily).toBe('Playfair Display');
    const custom = withTextStyle(kit, { fontFamily: 'Bebas Neue' });
    expect(withFont(custom, 'heading', 'Oswald').textStyle.fontFamily).toBe('Bebas Neue');
  });

  it('keeps the draft’s edits when logos are added on the server', () => {
    const draft = { ...libraryKit(2), name: 'Acme 2' };
    const saved = libraryKit(3, {
      logos: [
        { name: 'Logo', file: 'logos/a.png' },
        { name: 'Noir', file: 'logos/b.png' },
      ],
    });
    const merged = withAddedLogos(draft, saved);
    expect(merged.name).toBe('Acme 2');
    expect(merged.version).toBe(3);
    expect(merged.logos.map((logo) => logo.file)).toEqual(['logos/a.png', 'logos/b.png']);
    expect(isDirty(merged, saved)).toBe(true);
    expect(isDirty({ ...saved, version: 9 }, saved)).toBe(false);
  });
});

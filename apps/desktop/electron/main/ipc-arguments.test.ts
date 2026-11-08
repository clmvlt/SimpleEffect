import { describe, expect, it } from 'vitest';

import {
  parseChooseFileOptions,
  parseChooseFilesOptions,
  parseChooseFolderOptions,
  parseExternalUrl,
  parseNotification,
  parseRevealPath,
} from './ipc-arguments';

describe('parseChooseFolderOptions', () => {
  it('keeps valid options', () => {
    expect(parseChooseFolderOptions({ title: 'T', buttonLabel: 'B' })).toEqual({
      title: 'T',
      buttonLabel: 'B',
    });
    expect(
      parseChooseFolderOptions({ title: 'T', buttonLabel: 'B', defaultPath: '/home' }),
    ).toEqual({ title: 'T', buttonLabel: 'B', defaultPath: '/home' });
  });

  it('refuses malformed options', () => {
    expect(() => parseChooseFolderOptions(null)).toThrow(TypeError);
    expect(() => parseChooseFolderOptions({ title: 1, buttonLabel: 'B' })).toThrow(TypeError);
    expect(() =>
      parseChooseFolderOptions({ title: 'T', buttonLabel: 'B', defaultPath: 3 }),
    ).toThrow(TypeError);
  });
});

describe('parseChooseFilesOptions', () => {
  const valid = {
    title: 'Importer',
    buttonLabel: 'Importer',
    filterName: 'Médias',
    allFilesName: 'Tous les fichiers',
    extensions: ['mp4', 'wav'],
  };

  it('keeps valid options', () => {
    expect(parseChooseFilesOptions(valid)).toEqual(valid);
  });

  it('refuses malformed options and extensions', () => {
    expect(() => parseChooseFilesOptions(undefined)).toThrow(TypeError);
    expect(() => parseChooseFilesOptions({ ...valid, filterName: 1 })).toThrow(TypeError);
    expect(() => parseChooseFilesOptions({ ...valid, extensions: ['.mp4'] })).toThrow(TypeError);
    expect(() => parseChooseFilesOptions({ ...valid, extensions: 'mp4' })).toThrow(TypeError);
  });
});

describe('parseRevealPath', () => {
  it('accepts absolute paths only', () => {
    expect(parseRevealPath('/projects/Film.seproj')).toBe('/projects/Film.seproj');
    expect(() => parseRevealPath('Film.seproj')).toThrow(TypeError);
    expect(() => parseRevealPath(42)).toThrow(TypeError);
  });
});

describe('parseChooseFileOptions', () => {
  it('keeps valid options and refuses others', () => {
    expect(parseChooseFileOptions({ title: 'T', buttonLabel: 'B' })).toEqual({
      title: 'T',
      buttonLabel: 'B',
    });
    expect(() => parseChooseFileOptions({ title: 'T' })).toThrow(TypeError);
  });
});

describe('parseExternalUrl', () => {
  it('opens https pages only', () => {
    expect(parseExternalUrl('https://code.claude.com/docs/en/setup')).toBe(
      'https://code.claude.com/docs/en/setup',
    );
    for (const url of [
      'http://example.com',
      'file:///etc/passwd',
      'https://u:p@x.com',
      'nope',
      3,
    ]) {
      expect(() => parseExternalUrl(url)).toThrow(TypeError);
    }
  });
});

describe('parseNotification', () => {
  it('keeps short strings and refuses anything else', () => {
    const notification = { title: 'Tâche IA #3 terminée', body: 'Titre ajouté', tag: 'task-1' };
    expect(parseNotification(notification)).toEqual(notification);
    for (const value of [
      null,
      'text',
      { title: 'T', body: 'B' },
      { title: 'T', body: 3, tag: 'x' },
      { title: 'T'.repeat(501), body: 'B', tag: 'x' },
    ]) {
      expect(() => parseNotification(value)).toThrow(TypeError);
    }
  });
});

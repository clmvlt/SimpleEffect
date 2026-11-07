import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { describe, expect, it } from 'vitest';

import {
  describePlan,
  isUpToDate,
  parseArgs,
  platformKey,
  stampFor,
  validateManifest,
} from './binaries.mjs';
import { repoRoot } from './run-command.mjs';

const manifest = JSON.parse(
  readFileSync(resolve(repoRoot, 'scripts', 'binaries.manifest.json'), 'utf8'),
);

describe('the committed manifest', () => {
  it('pins every platform with https URLs, hashes, sizes and licences', () => {
    expect(() => validateManifest(manifest)).not.toThrow();
  });

  it('installs ffmpeg and ffprobe on every platform', () => {
    for (const [key, entry] of Object.entries(manifest.platforms)) {
      const names = entry.downloads.flatMap((download) => Object.keys(download.files));
      const suffix = key.startsWith('win32') ? '.exe' : '';
      expect(names.sort()).toEqual([`ffmpeg${suffix}`, `ffprobe${suffix}`]);
    }
  });
});

describe('validateManifest', () => {
  it('lists every problem', () => {
    const broken = structuredClone(manifest);
    broken.platforms['linux-x64'].downloads[0].sha256 = 'ABC';
    broken.platforms['darwin-x64'].downloads[0].url = 'http://example.com/ffmpeg.zip';
    broken.platforms['win32-x64'].downloads[0].files = { '../ffmpeg.exe': 'x' };
    expect(() => validateManifest(broken)).toThrow(
      /linux-x64\[0\]: sha256[\s\S]*darwin-x64\[0\]: url must be https|darwin-x64\[0\]: url[\s\S]*linux-x64/,
    );
    expect(() => validateManifest(broken)).toThrow(/plain file names/);
    expect(() => validateManifest({})).toThrow(/platforms/);
  });
});

describe('parseArgs', () => {
  it('reads --yes, --all and repeated --platform', () => {
    expect(parseArgs(['--yes', '--platform', 'linux-x64', '--platform', 'darwin-x64'])).toEqual({
      yes: true,
      all: false,
      platforms: ['linux-x64', 'darwin-x64'],
    });
    expect(parseArgs([])).toEqual({ yes: false, all: false, platforms: null });
  });

  it('refuses unknown arguments and platforms', () => {
    expect(() => parseArgs(['--force'])).toThrow(/unknown argument/);
    expect(() => parseArgs(['--platform', 'win32-arm64'])).toThrow(/expects one of/);
  });
});

describe('platforms and stamps', () => {
  it('maps Node platforms to folder keys', () => {
    expect(platformKey('win32', 'x64')).toBe('win32-x64');
    expect(platformKey('darwin', 'arm64')).toBe('darwin-arm64');
    expect(platformKey('linux', 'arm64')).toBeNull();
  });

  it('is up to date only when the stamp has the same hashes', () => {
    const entry = manifest.platforms['darwin-arm64'];
    expect(isUpToDate(stampFor(entry), entry)).toBe(true);
    expect(isUpToDate({ sha256: [entry.downloads[0].sha256] }, entry)).toBe(false);
    expect(isUpToDate(null, entry)).toBe(false);
  });

  it('describes downloads with sizes and sources', () => {
    const lines = describePlan(manifest, 'win32-x64');
    expect(lines[0]).toContain('FFmpeg 9.0.2');
    expect(lines[1]).toContain('114.8 MB');
    expect(lines[1]).toContain('https://github.com/GyanD/codexffmpeg');
  });
});

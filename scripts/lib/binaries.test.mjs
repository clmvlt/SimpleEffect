import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

import { describe, expect, it } from 'vitest';

import {
  buildCommands,
  buildHostProblem,
  describePlan,
  installedFolders,
  isPlainRelativePath,
  parseArgs,
  pendingDownloads,
  platformKey,
  stampFor,
  validateManifest,
} from './binaries.mjs';
import { repoRoot } from './run-command.mjs';

const manifest = JSON.parse(
  readFileSync(resolve(repoRoot, 'scripts', 'binaries.manifest.json'), 'utf8'),
);

/** The installed names of a platform. @param {string} key */
const installedNames = (key) =>
  manifest.platforms[key].downloads.flatMap((download) => Object.keys(download.files));

/** The download of a platform that installs into `whisper/`. @param {string} key */
const whisperDownload = (key) =>
  manifest.platforms[key].downloads.find((download) =>
    installedFolders(download).includes('whisper'),
  );

describe('the committed manifest', () => {
  it('pins every platform with https URLs, hashes, sizes and licences', () => {
    expect(() => validateManifest(manifest)).not.toThrow();
  });

  it('installs ffmpeg and ffprobe at the top of every platform folder', () => {
    for (const key of Object.keys(manifest.platforms)) {
      const suffix = key.startsWith('win32') ? '.exe' : '';
      const topLevel = installedNames(key).filter((name) => !name.includes('/'));
      expect(topLevel.sort()).toEqual([`ffmpeg${suffix}`, `ffprobe${suffix}`]);
    }
  });

  it('installs whisper-cli in whisper/ on every platform, with its libraries', () => {
    for (const key of Object.keys(manifest.platforms)) {
      const suffix = key.startsWith('win32') ? '.exe' : '';
      expect(installedNames(key)).toContain(`whisper/whisper-cli${suffix}`);
    }
    expect(installedNames('win32-x64')).toContain('whisper/ggml-cpu-x64.dll');
    expect(installedNames('linux-x64')).toContain('whisper/libwhisper.so.1');
  });

  it('installs the ONNX Runtime library in onnxruntime/ except on Intel Macs', () => {
    expect(installedNames('win32-x64')).toContain('onnxruntime/onnxruntime.dll');
    expect(installedNames('darwin-arm64')).toContain('onnxruntime/libonnxruntime.dylib');
    expect(installedNames('linux-x64')).toContain('onnxruntime/libonnxruntime.so');
    expect(installedNames('darwin-x64').some((name) => name.startsWith('onnxruntime/'))).toBe(
      false,
    );
  });

  it('compiles whisper.cpp from source on macOS only, for the matching architecture', () => {
    expect(whisperDownload('win32-x64').build).toBeUndefined();
    expect(whisperDownload('linux-x64').build).toBeUndefined();
    for (const [key, arch] of [
      ['darwin-arm64', 'arm64'],
      ['darwin-x64', 'x86_64'],
    ]) {
      const { build } = whisperDownload(key);
      expect(build.system).toBe('cmake');
      expect(build.steps.flat()).toContain(`-DCMAKE_OSX_ARCHITECTURES=${arch}`);
      expect(build.steps.flat()).toContain('-DGGML_METAL_EMBED_LIBRARY=ON');
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
    expect(() => validateManifest(broken)).toThrow(/installed names must be relative paths/);
    expect(() => validateManifest({})).toThrow(/platforms/);
  });

  it('requires the versions and the whisper.cpp licence', () => {
    const broken = structuredClone(manifest);
    delete broken.whisperVersion;
    broken.whisperLicense = '';
    expect(() => validateManifest(broken)).toThrow(/whisperVersion[\s\S]*whisperLicense/);
  });

  it('checks the shape of source builds', () => {
    const broken = structuredClone(manifest);
    const download = broken.platforms['darwin-arm64'].downloads[2];
    download.build = { system: 'make', source: '../src', steps: [['-S', '{src}'], []] };
    const message = () => validateManifest(broken);
    expect(message).toThrow(/darwin-arm64\[2\]: build\.system must be one of cmake/);
    expect(message).toThrow(/build\.source must be/);
    expect(message).toThrow(/build\.steps must be a non-empty list/);
    download.build = { system: 'cmake', source: 'src', steps: [['-S', '{src}', '-j', '{jobs}']] };
    expect(message).toThrow(/unknown placeholders \{src\}/);
  });

  it('refuses two downloads sharing an installed name or a subfolder', () => {
    const broken = structuredClone(manifest);
    broken.platforms['win32-x64'].downloads[0].files['whisper/other.dll'] = 'other.dll';
    expect(() => validateManifest(broken)).toThrow(/win32-x64: downloads 0 and 1 both install/);
  });
});

describe('isPlainRelativePath', () => {
  it('accepts file names and subfolder paths', () => {
    expect(isPlainRelativePath('ffmpeg.exe')).toBe(true);
    expect(isPlainRelativePath('whisper/whisper-cli.exe')).toBe(true);
    expect(isPlainRelativePath('whisper/libggml-base.so.0')).toBe(true);
    expect(isPlainRelativePath('whisper.cpp-1.9.5')).toBe(true);
  });

  it('refuses parent references, absolute paths, backslashes and empty segments', () => {
    for (const path of [
      '../ffmpeg',
      'whisper/../ffmpeg',
      './ffmpeg',
      '/usr/bin/ffmpeg',
      'C:/ffmpeg.exe',
      'whisper\\whisper-cli.exe',
      'whisper//whisper-cli',
      'whisper/',
      '',
      '.fetched.json',
      undefined,
    ]) {
      expect(isPlainRelativePath(path), String(path)).toBe(false);
    }
  });
});

describe('source builds', () => {
  const build = whisperDownload('darwin-arm64').build;

  it('replaces the placeholders of every step', () => {
    const commands = buildCommands(build, { source: '/w/src', build: '/w/src/build', jobs: 8 });
    expect(commands.map((command) => command.program)).toEqual(['cmake', 'cmake']);
    expect(commands[0]?.args.slice(0, 4)).toEqual(['-S', '/w/src', '-B', '/w/src/build']);
    expect(commands[1]?.args).toEqual(
      expect.arrayContaining(['--build', '/w/src/build', '--target', 'whisper-cli', '8']),
    );
    expect(commands.flatMap((command) => command.args).join(' ')).not.toMatch(/[{}]/);
  });

  it('runs only on the operating system of the platform', () => {
    const downloads = manifest.platforms['darwin-x64'].downloads;
    expect(buildHostProblem('darwin-x64', downloads, 'darwin')).toBeNull();
    expect(buildHostProblem('darwin-x64', downloads, 'win32')).toMatch(
      /darwin-x64 compiles whisper\.cpp-1\.9\.5 from source[\s\S]*on macOS/,
    );
    expect(buildHostProblem('darwin-x64', downloads.slice(0, 2), 'win32')).toBeNull();
    expect(buildHostProblem('linux-x64', manifest.platforms['linux-x64'].downloads, 'win32')).toBe(
      null,
    );
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

  it('fetches only the downloads the stamp does not record', () => {
    const entry = manifest.platforms['darwin-arm64'];
    expect(pendingDownloads(stampFor(entry), entry)).toEqual([]);
    expect(pendingDownloads(null, entry)).toEqual(entry.downloads);
    const ffmpegOnly = { sha256: [entry.downloads[0].sha256, entry.downloads[1].sha256] };
    expect(pendingDownloads(ffmpegOnly, entry)).toEqual(entry.downloads.slice(2));
  });

  it('re-fetches a download whose hash changed', () => {
    const entry = structuredClone(manifest.platforms['win32-x64']);
    const stamp = stampFor(entry);
    entry.downloads[1].sha256 = 'f'.repeat(64);
    expect(pendingDownloads(stamp, entry)).toEqual([entry.downloads[1]]);
  });

  it('stamps the installed downloads in manifest order', () => {
    const entry = manifest.platforms['win32-x64'];
    const [ffmpeg, whisper, onnxRuntime] = entry.downloads.map((download) => download.sha256);
    expect(stampFor(entry)).toEqual({ sha256: [ffmpeg, whisper, onnxRuntime] });
    expect(stampFor(entry, new Set([whisper, 'gone']))).toEqual({ sha256: [whisper] });
  });

  it('describes downloads with products, licences, sizes and sources', () => {
    const lines = describePlan(manifest, 'win32-x64');
    expect(lines.join('\n')).toContain('FFmpeg 9.0.2: GPL-3.0-or-later');
    expect(lines.join('\n')).toContain('whisper.cpp 1.9.5: MIT');
    expect(lines.join('\n')).toContain('ONNX Runtime 1.31.0: MIT');
    expect(lines.find((line) => line.includes('codexffmpeg'))).toContain('114.8 MB');
    expect(lines.find((line) => line.includes('whisper-bin-x64.zip'))).toContain('8.9 MB');
    expect(lines.find((line) => line.includes('onnxruntime-win-x64'))).toContain('82.9 MB');
    expect(lines.at(-1)).toContain('total 206.6 MB');
  });

  it('describes only the given downloads and marks source builds', () => {
    const entry = manifest.platforms['darwin-arm64'];
    const lines = describePlan(manifest, 'darwin-arm64', [entry.downloads[2]]);
    expect(lines.filter((line) => line.includes(' MB  https://'))).toHaveLength(1);
    expect(lines.join('\n')).toContain('(source, compiled here with cmake)');
    expect(lines.at(-1)).toContain('total 9.6 MB');
  });
});

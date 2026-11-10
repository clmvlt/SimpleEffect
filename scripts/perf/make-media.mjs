// Writes the synthetic media of the performance measures (prompt 29, docs/performance.md) into
// target/perf-media/ with the pinned ffmpeg (`pnpm fetch-binaries`): 4K H.264 (8-bit, 30 fps)
// and 4K HEVC Main 10 (10-bit, 30 fps), 30 s each, with moving detail (testsrc2 + noise) so
// decoders work as they do on real footage, and a tone. Existing files are kept.
//
//   node scripts/perf/make-media.mjs [--seconds 30]
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { platformKey } from '../lib/binaries.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..', '..');
const output = join(root, 'target', 'perf-media');

function ffmpegPath() {
  const key = platformKey(process.platform, process.arch);
  const executable = process.platform === 'win32' ? 'ffmpeg.exe' : 'ffmpeg';
  const path = key === null ? null : join(root, 'resources', 'bin', key, executable);
  if (path === null || !existsSync(path)) {
    throw new Error('ffmpeg not found: run `pnpm fetch-binaries` first.');
  }
  return path;
}

function secondsArgument() {
  const index = process.argv.indexOf('--seconds');
  const value = index === -1 ? 30 : Number(process.argv[index + 1]);
  if (!Number.isInteger(value) || value < 1 || value > 600) {
    throw new Error('--seconds takes a whole number of seconds between 1 and 600.');
  }
  return value;
}

/** A 4K picture with detail everywhere and a tone, as ffmpeg input arguments. */
function source(seconds) {
  return [
    '-f',
    'lavfi',
    '-i',
    `testsrc2=s=3840x2160:r=30:d=${seconds},noise=alls=3:allf=t`,
    '-f',
    'lavfi',
    '-i',
    `sine=frequency=440:sample_rate=48000:duration=${seconds}`,
  ];
}

const FILES = [
  {
    name: '4k-h264.mp4',
    codec: ['-c:v', 'libx264', '-preset', 'veryfast', '-crf', '23', '-pix_fmt', 'yuv420p'],
  },
  {
    name: '4k-hevc-10bit.mp4',
    codec: [
      '-c:v',
      'libx265',
      '-preset',
      'veryfast',
      '-crf',
      '22',
      '-pix_fmt',
      'yuv420p10le',
      '-tag:v',
      'hvc1',
    ],
  },
];

const ffmpeg = ffmpegPath();
const seconds = secondsArgument();
mkdirSync(output, { recursive: true });
for (const file of FILES) {
  const path = join(output, file.name);
  if (existsSync(path)) {
    console.log(`kept ${path}`);
    continue;
  }
  console.log(`writing ${path}…`);
  const result = spawnSync(
    ffmpeg,
    [
      '-hide_banner',
      '-loglevel',
      'error',
      ...source(seconds),
      ...file.codec,
      '-g',
      '60',
      '-c:a',
      'aac',
      '-b:a',
      '128k',
      '-movflags',
      '+faststart',
      path,
    ],
    { stdio: 'inherit' },
  );
  if (result.status !== 0) throw new Error(`ffmpeg could not write ${file.name}.`);
}

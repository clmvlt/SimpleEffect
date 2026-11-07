// Writes the media fixtures of the engine's golden-frame tests into
// packages/engine/test/golden/fixtures/ with the pinned ffmpeg (`pnpm fetch-binaries`).
// The fixtures are committed (a few kilobytes each), so tests never need ffmpeg: re-run this
// script only to change them, then refresh the reference images (`SE_UPDATE_GOLDEN=1`).
//
// Every video encodes its frame number N in binary in its top half: eight columns, column k is
// white when bit k of N is set (k = 0 on the left), so tests read the decoded frame number back.
// The bottom half shows a bar moving 3 px per frame (at 320 px wide) for a visible check.
import { spawnSync } from 'node:child_process';
import { existsSync, mkdirSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

import { platformKey } from './lib/binaries.mjs';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const output = join(root, 'packages', 'engine', 'test', 'golden', 'fixtures');

const FRAME_NUMBER_LUMA =
  'if(lt(Y,H/2),if(mod(floor(N/pow(2,floor(X*8/W))),2),235,16),if(lt(abs(X-mod(N*3*W/320,W)),2*W/160),235,80))';

/** H.264 tags so every decoder converts YUV to RGB the same way. */
const BT709 = [
  '-colorspace',
  'bt709',
  '-color_primaries',
  'bt709',
  '-color_trc',
  'bt709',
  '-color_range',
  'tv',
];

function ffmpegPath() {
  const key = platformKey(process.platform, process.arch);
  const executable = process.platform === 'win32' ? 'ffmpeg.exe' : 'ffmpeg';
  const path = key === null ? null : join(root, 'resources', 'bin', key, executable);
  if (path === null || !existsSync(path)) {
    throw new Error('ffmpeg not found: run `pnpm fetch-binaries` first.');
  }
  return path;
}

/** A counting picture source of `size` at `rate` for `seconds`, as ffmpeg input arguments. */
function countingSource(size, rate, seconds) {
  return [
    '-f',
    'lavfi',
    '-i',
    `color=c=black:s=${size}:r=${rate}:d=${seconds}`,
    '-vf',
    `format=yuv420p,geq=lum='${FRAME_NUMBER_LUMA}':cb=128:cr=128`,
  ];
}

const H264 = [
  '-c:v',
  'libx264',
  '-profile:v',
  'high',
  '-preset',
  'medium',
  '-crf',
  '12',
  '-bf',
  '3',
  '-g',
  '30',
  '-pix_fmt',
  'yuv420p',
];

const fixtures = [
  // 29.97 fps H.264 with B-frames in MP4: the frame-accuracy case (composition offsets, edit list).
  [
    'count-2997.mp4',
    [...countingSource('320x180', '30000/1001', 3), ...H264, ...BT709, '-movflags', '+faststart'],
  ],
  // Its proxy: the same pictures at half resolution.
  [
    'count-2997-proxy.mp4',
    [...countingSource('160x90', '30000/1001', 3), ...H264, ...BT709, '-movflags', '+faststart'],
  ],
  // 25 fps VP9 in WebM: another codec, container and rate.
  [
    'count-25.webm',
    [
      ...countingSource('320x180', '25', 2),
      '-c:v',
      'libvpx-vp9',
      '-crf',
      '20',
      '-b:v',
      '0',
      '-g',
      '25',
      ...BT709,
    ],
  ],
  // SMPTE HD colour bars: the test pattern.
  ['bars.png', ['-f', 'lavfi', '-i', 'smptehdbars=s=320x180', '-frames:v', '1']],
  // A translucent-edged yellow disc with alpha: compositing over lower tracks.
  [
    'disc.png',
    [
      '-f',
      'lavfi',
      '-i',
      'color=c=black@0:s=64x64',
      '-vf',
      "format=rgba,geq=r='255':g='200':b='0':a='if(lt(hypot(X-31.5,Y-31.5),24),255,if(lt(hypot(X-31.5,Y-31.5),30),128,0))'",
      '-frames:v',
      '1',
    ],
  ],
];

function run(ffmpeg, args) {
  const result = spawnSync(ffmpeg, ['-hide_banner', '-loglevel', 'error', '-y', ...args], {
    stdio: 'inherit',
  });
  if (result.status !== 0) {
    throw new Error(`ffmpeg failed for ${args.at(-1)}`);
  }
}

const ffmpeg = ffmpegPath();
mkdirSync(output, { recursive: true });
for (const [name, args] of fixtures) {
  run(ffmpeg, [...args, '-map_metadata', '-1', join(output, name)]);
  console.log(`wrote ${name}`);
}
// Phone footage: the 29.97 clip with a display matrix turning it 90° clockwise
// (ffmpeg's -display_rotation is counter-clockwise).
run(ffmpeg, [
  '-display_rotation:v:0',
  '270',
  '-i',
  join(output, 'count-2997.mp4'),
  '-c',
  'copy',
  '-map_metadata',
  '-1',
  join(output, 'count-2997-rotated.mp4'),
]);
console.log('wrote count-2997-rotated.mp4');

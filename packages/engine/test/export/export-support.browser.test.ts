/**
 * The export dialog's encoder probe (prompt 27, ADR-0038) against real WebCodecs: masters go to
 * ffmpeg, PCM needs no encoder, and codecs the headless Chromium encodes are reported as such.
 */
import type { VideoExportSettings } from '@se/protocol';
import { describe, expect, it } from 'vitest';

import { audioEncoderSupport, videoEncoderSupport } from '../../src/export/export-support';

const VP9: VideoExportSettings = {
  codec: 'vp9',
  width: 320,
  height: 180,
  rate: { kind: 'quality', level: 'high' },
};

describe('videoEncoderSupport', () => {
  it('has nothing to probe without a picture and sends masters to ffmpeg', async () => {
    expect(await videoEncoderSupport(null, 25)).toBeNull();
    expect(await videoEncoderSupport({ ...VP9, codec: 'prores422Hq' }, 25)).toBe('ffmpeg');
    expect(await videoEncoderSupport({ ...VP9, codec: 'dnxhrHq' }, 25)).toBe('ffmpeg');
  });

  it('finds an encoder for a codec Chromium ships', async () => {
    expect(['hardware', 'software']).toContain(await videoEncoderSupport(VP9, 25));
  });

  it('finds none for a size no encoder takes', async () => {
    expect(await videoEncoderSupport({ ...VP9, width: 65_536, height: 65_536 }, 25)).toBe('none');
  });
});

describe('audioEncoderSupport', () => {
  it('needs no encoder for PCM and finds Opus', async () => {
    expect(await audioEncoderSupport(null)).toBeNull();
    const pcm = { codec: 'pcm', bitsPerSecond: 0, encoder: 'webCodecs' } as const;
    expect(await audioEncoderSupport(pcm)).toBe('webCodecs');
    const opus = { codec: 'opus', bitsPerSecond: 128_000, encoder: 'webCodecs' } as const;
    expect(await audioEncoderSupport(opus)).toBe('webCodecs');
  });
});

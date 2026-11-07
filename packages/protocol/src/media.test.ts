import { describe, expect, it } from 'vitest';

import { assetMediaPath, mediaUrl } from './media-url';
import { decodeWaveformPeaks } from './waveform-peaks';

const ENDPOINT = { url: 'http://127.0.0.1:4000', token: 'secret' };

describe('media URLs', () => {
  it('names every asset file', () => {
    const id = '0193e7d2-0000-7000-8000-000000000001';
    expect(assetMediaPath(id, 'original')).toBe(`/media/assets/${id}/original`);
    expect(assetMediaPath(id, 'mezzanine')).toBe(`/media/assets/${id}/mezzanine`);
    expect(assetMediaPath(id, 'proxy')).toBe(`/media/assets/${id}/proxy`);
    expect(assetMediaPath(id, 'thumbnails')).toBe(`/media/assets/${id}/thumbnails`);
    expect(assetMediaPath(id, 'thumbnailSheet')).toBe(`/media/assets/${id}/thumbnails/sheet`);
    expect(assetMediaPath(id, 'poster')).toBe(`/media/assets/${id}/thumbnails/poster`);
    expect(assetMediaPath(id, { peaks: 1024 })).toBe(`/media/assets/${id}/peaks/1024`);
  });

  it('carries the token and an optional version', () => {
    expect(mediaUrl(ENDPOINT, '/media/assets/a/original')).toBe(
      'http://127.0.0.1:4000/media/assets/a/original?t=secret',
    );
    expect(mediaUrl(ENDPOINT, '/media/assets/a/poster', 3)).toBe(
      'http://127.0.0.1:4000/media/assets/a/poster?t=secret&v=3',
    );
  });
});

/** A SEPK file as se-media writes it (header of 32 bytes, then Int16 pairs). */
function sepk(channels: number, values: number[]): ArrayBuffer {
  const peakCount = values.length / (channels * 2);
  const buffer = new ArrayBuffer(32 + values.length * 2);
  const view = new DataView(buffer);
  for (let index = 0; index < 4; index += 1) {
    view.setUint8(index, 'SEPK'.charCodeAt(index));
  }
  view.setUint16(4, 1, true);
  view.setUint16(6, channels, true);
  view.setUint32(8, 48_000, true);
  view.setUint32(12, 256, true);
  view.setUint32(16, peakCount, true);
  values.forEach((value, index) => {
    view.setInt16(32 + index * 2, value, true);
  });
  return buffer;
}

describe('decodeWaveformPeaks', () => {
  it('reads the header and the values', () => {
    const peaks = decodeWaveformPeaks(sepk(2, [-5, 9, 0, 0, -32767, 32767, 1, 2]));
    expect(peaks).toMatchObject({ channels: 2, sampleRate: 48_000, samplesPerPeak: 256 });
    expect(peaks.peakCount).toBe(2);
    expect([...peaks.values]).toEqual([-5, 9, 0, 0, -32767, 32767, 1, 2]);
  });

  it('refuses other files', () => {
    expect(() => decodeWaveformPeaks(new ArrayBuffer(8))).toThrow(/shorter/);
    const wrongMagic = sepk(1, [0, 0]);
    new DataView(wrongMagic).setUint8(0, 0x52);
    expect(() => decodeWaveformPeaks(wrongMagic)).toThrow(/not a SEPK/);
    expect(() => decodeWaveformPeaks(sepk(1, [0, 0]).slice(0, 34))).toThrow(/does not match/);
  });
});

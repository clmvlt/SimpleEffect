import type { Asset, Flicks } from '@se/protocol';

/** A probed 1080p H.264 video asset waiting for its decodability check, with `overrides`. */
export function testAsset(overrides: Partial<Asset>): Asset {
  return {
    id: '0193e7d2-0000-7000-8000-000000000001',
    originalPath: '/rushes/clip.mp4',
    relativeHint: '../rushes/clip.mp4',
    kind: 'video',
    mediaInfo: {
      duration: (10 * 705_600_000) as Flicks,
      container: 'mov,mp4,m4a,3gp,3g2,mj2',
      videoStream: {
        codec: 'h264',
        profile: 'High',
        level: 40,
        codecTag: 'avc1',
        pixelFormat: 'yuv420p',
        bitDepth: 8,
        chromaSubsampling: '4:2:0',
        width: 1920,
        height: 1080,
        frameRate: { numerator: 30, denominator: 1 },
        rotation: 0,
      },
      audioStreams: [{ codec: 'aac', profile: 'LC', channels: 2, sampleRate: 48_000 }],
    },
    fingerprint: { sizeBytes: 1000, modifiedUnixMs: 1, sampleHash: '0000000000000001' },
    derived: { mezzanine: null, proxy: null },
    decodability: { video: 'unknown', audio: 'unknown' },
    missing: false,
    ...overrides,
  };
}

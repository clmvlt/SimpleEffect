import type { AudioStreamInfo, MediaInfo, VideoStreamInfo } from '@se/protocol';
import { describe, expect, it, vi } from 'vitest';

import {
  audioCodecString,
  checkDecodability,
  checkDecoderCapabilities,
  videoCodecString,
} from './index';
import type { CodecSupport } from './index';

function video(overrides: Partial<VideoStreamInfo>): VideoStreamInfo {
  return {
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
    ...overrides,
  };
}

function audio(overrides: Partial<AudioStreamInfo>): AudioStreamInfo {
  return { codec: 'aac', profile: 'LC', channels: 2, sampleRate: 48_000, ...overrides };
}

describe('videoCodecString', () => {
  it('builds avc1 strings from profile and level', () => {
    expect(videoCodecString(video({}))).toBe('avc1.640028');
    expect(videoCodecString(video({ profile: 'Constrained Baseline', level: 31 }))).toBe(
      'avc1.42401F',
    );
    expect(videoCodecString(video({ profile: 'High 4:2:2', level: 51, bitDepth: 10 }))).toBe(
      'avc1.7A0033',
    );
  });

  it('builds hvc1 strings for Main and Main 10, keeping hev1 tags', () => {
    expect(videoCodecString(video({ codec: 'hevc', profile: 'Main', level: 93 }))).toBe(
      'hvc1.1.6.L93.B0',
    );
    expect(
      videoCodecString(
        video({ codec: 'hevc', profile: 'Main 10', level: 120, codecTag: 'hev1', bitDepth: 10 }),
      ),
    ).toBe('hev1.2.4.L120.B0');
  });

  it('builds vp09 and av01 strings with the bit depth', () => {
    expect(
      videoCodecString(video({ codec: 'vp9', profile: 'Profile 2', level: null, bitDepth: 10 })),
    ).toBe('vp09.02.41.10');
    expect(videoCodecString(video({ codec: 'av1', profile: 'Main', level: 8, bitDepth: 10 }))).toBe(
      'av01.0.08M.10',
    );
  });

  it('guesses a level from the size when the prober gave none', () => {
    expect(videoCodecString(video({ level: null, width: 3840, height: 2160 }))).toBe('avc1.640033');
  });

  it('has no string for codecs WebCodecs cannot decode', () => {
    expect(videoCodecString(video({ codec: 'prores', profile: 'HQ' }))).toBeNull();
    expect(videoCodecString(video({ codec: 'dnxhd' }))).toBeNull();
  });
});

describe('audioCodecString', () => {
  it('maps AAC profiles and other codecs', () => {
    expect(audioCodecString(audio({}))).toBe('mp4a.40.2');
    expect(audioCodecString(audio({ profile: 'HE-AAC' }))).toBe('mp4a.40.5');
    expect(audioCodecString(audio({ codec: 'opus', profile: null }))).toBe('opus');
    expect(audioCodecString(audio({ codec: 'pcm_s24le', profile: null }))).toBe('pcm-s24');
    expect(audioCodecString(audio({ codec: 'pcm_s24be', profile: null }))).toBeNull();
  });
});

describe('checkDecodability', () => {
  const info: MediaInfo = {
    duration: null,
    container: 'mov,mp4,m4a,3gp,3g2,mj2',
    videoStream: video({}),
    audioStreams: [audio({})],
  };

  it('asks the decoders with the codec strings', async () => {
    const support = {
      VideoDecoder: { isConfigSupported: vi.fn(() => Promise.resolve({ supported: true })) },
      AudioDecoder: { isConfigSupported: vi.fn(() => Promise.resolve({ supported: false })) },
    } satisfies CodecSupport;
    const result = await checkDecodability('video', info, support);
    expect(result).toEqual({ video: 'decodable', audio: 'undecodable' });
    expect(support.VideoDecoder.isConfigSupported).toHaveBeenCalledWith({
      codec: 'avc1.640028',
      codedWidth: 1920,
      codedHeight: 1080,
    });
    expect(support.AudioDecoder.isConfigSupported).toHaveBeenCalledWith({
      codec: 'mp4a.40.2',
      sampleRate: 48_000,
      numberOfChannels: 2,
    });
  });

  it('counts unknown codecs, thrown errors and missing APIs as undecodable', async () => {
    const throwing: CodecSupport = {
      VideoDecoder: { isConfigSupported: () => Promise.reject(new TypeError('bad codec')) },
    };
    const prores = { ...info, videoStream: video({ codec: 'prores' }) };
    expect(await checkDecodability('video', prores, throwing)).toEqual({
      video: 'undecodable',
      audio: 'undecodable',
    });
    expect((await checkDecodability('video', info, throwing)).video).toBe('undecodable');
  });

  it('reports absent streams and checks images by type', async () => {
    const image: MediaInfo = {
      ...info,
      videoStream: video({ codec: 'png', profile: null }),
      audioStreams: [],
    };
    const support: CodecSupport = {
      ImageDecoder: { isTypeSupported: (type) => Promise.resolve(type === 'image/png') },
    };
    expect(await checkDecodability('image', image, support)).toEqual({
      video: 'decodable',
      audio: 'noStream',
    });
    const sound: MediaInfo = { ...info, videoStream: null };
    expect((await checkDecodability('audio', sound, {})).video).toBe('noStream');
  });
});

describe('checkDecoderCapabilities', () => {
  it('asks for 4K HEVC Main 10, H.264 High, AAC and FLAC', async () => {
    const videoConfigs: string[] = [];
    const support: CodecSupport = {
      VideoDecoder: {
        isConfigSupported: (config) => {
          videoConfigs.push(config.codec);
          return Promise.resolve({ supported: config.codec.startsWith('avc1') });
        },
      },
      AudioDecoder: { isConfigSupported: () => Promise.reject(new TypeError('no audio')) },
    };
    expect(await checkDecoderCapabilities(support)).toEqual({
      hevcMain10: false,
      h264High: true,
      aac: false,
      flac: false,
    });
    expect(videoConfigs).toEqual(['hvc1.2.4.L153.B0', 'avc1.640033']);
    expect(await checkDecoderCapabilities({})).toEqual({
      hevcMain10: false,
      h264High: false,
      aac: false,
      flac: false,
    });
  });
});

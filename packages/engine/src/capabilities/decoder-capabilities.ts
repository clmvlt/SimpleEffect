import type { DecoderCapabilities } from '@se/protocol';

import type { CodecSupport } from './decodability';

/** Configurations probed for each capability (4K, the largest size mezzanines and proxies keep). */
const PROBES = {
  hevcMain10: { codec: 'hvc1.2.4.L153.B0', codedWidth: 3840, codedHeight: 2160 },
  h264High: { codec: 'avc1.640033', codedWidth: 3840, codedHeight: 2160 },
  aac: { codec: 'mp4a.40.2', sampleRate: 48_000, numberOfChannels: 2 },
  flac: { codec: 'flac', sampleRate: 48_000, numberOfChannels: 2 },
} as const;

async function supported(check: () => Promise<{ supported?: boolean | undefined }> | undefined) {
  try {
    return (await check())?.supported === true;
  } catch {
    return false;
  }
}

/**
 * What this machine's WebCodecs decodes among the codecs of mezzanines and proxies (HEVC Main 10,
 * H.264 High, AAC, FLAC), sent to se-core as `reportDecoderCapabilities` before decodability
 * reports. Never rejects: refusals, thrown errors and missing APIs are `false`.
 */
export async function checkDecoderCapabilities(
  support: CodecSupport = globalThis,
): Promise<DecoderCapabilities> {
  const video = (config: VideoDecoderConfig) =>
    supported(() => support.VideoDecoder?.isConfigSupported(config));
  const audio = (config: AudioDecoderConfig) =>
    supported(() => support.AudioDecoder?.isConfigSupported(config));
  const [hevcMain10, h264High, aac, flac] = await Promise.all([
    video(PROBES.hevcMain10),
    video(PROBES.h264High),
    audio(PROBES.aac),
    audio(PROBES.flac),
  ]);
  return { hevcMain10, h264High, aac, flac };
}

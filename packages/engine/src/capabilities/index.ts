/**
 * `@se/engine/capabilities`: what this machine's WebCodecs can decode (`docs/architecture.md` §7).
 *
 * - `videoCodecString` / `audioCodecString`: codec strings built from probed `MediaInfo`
 *   (avc1, hvc1/hev1, vp09, av01, vp8; mp4a, opus, flac, PCM…).
 * - `checkDecodability`: asks `VideoDecoder` / `AudioDecoder` / `ImageDecoder` and returns one
 *   verdict per stream, the payload of the `reportDecodability` command.
 * - `checkDecoderCapabilities`: whether the codecs of mezzanines and proxies decode here (HEVC
 *   Main 10, H.264 High, AAC, FLAC), the payload of `reportDecoderCapabilities`.
 */
export { audioCodecString, videoCodecString } from './codec-strings';
export { checkDecodability } from './decodability';
export { checkDecoderCapabilities } from './decoder-capabilities';
export type { CodecSupport, DecodabilityResult } from './decodability';

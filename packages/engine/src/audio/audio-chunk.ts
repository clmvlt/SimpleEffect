import { CHUNK_SECONDS } from './audio-plan';

/** Decoded sound of one chunk of a file: `buffer` starts exactly at media second `start`. */
export interface AudioChunk {
  readonly buffer: AudioBuffer;
  readonly start: number;
}

/** A decoded audio buffer and its container timestamp (Mediabunny's `WrappedAudioBuffer`). */
export interface TimedAudioBuffer {
  readonly buffer: AudioBuffer;
  /** Container seconds of its first sample. */
  readonly timestamp: number;
}

/** Creates an empty `AudioBuffer` (`new AudioBuffer(…)` in browsers; a fake in Node tests). */
export type AudioBufferFactory = (
  channels: number,
  length: number,
  sampleRate: number,
) => AudioBuffer;

/**
 * Copies the decoded buffers overlapping chunk `index` into one buffer starting exactly at its
 * first sample (media time `index × CHUNK_SECONDS`, container time `origin` + that). Positions are
 * whole samples computed from each buffer's timestamp, so consecutive chunks join without gap or
 * overlap; a mono source gives a mono chunk (Web Audio spreads it to both speakers). The last
 * chunk of a file is shorter; `null` when no sample falls in the chunk.
 */
export function assembleChunk(
  buffers: readonly TimedAudioBuffer[],
  index: number,
  origin: number,
  createBuffer: AudioBufferFactory,
): AudioChunk | null {
  const first = buffers[0];
  if (first === undefined) return null;
  const { sampleRate } = first.buffer;
  const chunkStart = Math.round(index * CHUNK_SECONDS * sampleRate);
  const placed = buffers.map((timed) => ({
    buffer: timed.buffer,
    at: Math.round((timed.timestamp - origin) * sampleRate) - chunkStart,
  }));
  const fullLength = Math.round(CHUNK_SECONDS * sampleRate);
  const end = Math.max(...placed.map(({ buffer, at }) => at + buffer.length));
  const length = Math.min(fullLength, end);
  if (length <= 0) return null;
  const channels = Math.max(...placed.map(({ buffer }) => buffer.numberOfChannels));
  const chunk = createBuffer(channels, length, sampleRate);
  for (const { buffer, at } of placed) {
    const skip = Math.max(0, -at);
    const count = Math.min(buffer.length - skip, length - Math.max(0, at));
    if (count <= 0) continue;
    for (let channel = 0; channel < channels; channel++) {
      const source = buffer.getChannelData(Math.min(channel, buffer.numberOfChannels - 1));
      chunk.copyToChannel(source.subarray(skip, skip + count), channel, Math.max(0, at));
    }
  }
  return { buffer: chunk, start: index * CHUNK_SECONDS };
}

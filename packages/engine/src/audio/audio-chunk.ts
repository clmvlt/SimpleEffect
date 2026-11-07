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

/**
 * Where the last buffer of a chunk was placed (media sample index from media time 0), so the next
 * chunk of the file continues it exactly.
 */
export interface ChunkPlacement {
  readonly position: number;
  readonly length: number;
}

/** A chunk and the placement its successor continues. */
export interface AssembledChunk {
  readonly chunk: AudioChunk;
  readonly last: ChunkPlacement;
}

/** Creates an empty `AudioBuffer` (`new AudioBuffer(…)` in browsers; a fake in Node tests). */
export type AudioBufferFactory = (
  channels: number,
  length: number,
  sampleRate: number,
) => AudioBuffer;

/**
 * Containers may round timestamps to the millisecond (Matroska, WebM): a buffer stamped within
 * this many seconds (plus a sample) of where the previous one ended continues it.
 */
const TIMESTAMP_ROUNDING_SECONDS = 0.001;

/**
 * Media sample index of each buffer: decoded sound is contiguous, so a buffer continues the
 * previous one unless its timestamp says there is a real gap — Matroska and WebM round timestamps
 * to the millisecond, which placed one by one would leave clicks at every buffer.
 *
 * Each chunk decodes again the buffer straddling its start, which the previous chunk decoded
 * last; the decoder may stamp it differently in the two runs (rounded container timestamp in one,
 * accumulated sample count in the other). Recognised within the rounding tolerance, it takes the
 * position the previous chunk gave it, so the two chunks join on the sample.
 */
function placements(
  buffers: readonly TimedAudioBuffer[],
  origin: number,
  previous: ChunkPlacement | null,
): number[] {
  const first = buffers[0];
  if (first === undefined) return [];
  const { sampleRate } = first.buffer;
  const tolerance = Math.ceil(TIMESTAMP_ROUNDING_SECONDS * sampleRate) + 1;
  const near = (stamped: number, position: number) => Math.abs(stamped - position) <= tolerance;
  let expected: number | null = previous === null ? null : previous.position + previous.length;
  return buffers.map((timed, index) => {
    const stamped = Math.round((timed.timestamp - origin) * sampleRate);
    let at = stamped;
    if (index === 0 && previous !== null && near(stamped, previous.position))
      at = previous.position;
    else if (expected !== null && near(stamped, expected)) at = expected;
    expected = at + timed.buffer.length;
    return at;
  });
}

/**
 * Copies the decoded buffers overlapping chunk `index` into one buffer starting exactly at its
 * first sample (media time `index × CHUNK_SECONDS`, container time `origin` + that). Consecutive
 * buffers and consecutive chunks of a file join without gap or overlap (see `placements`); a mono
 * source gives a mono chunk (Web Audio spreads it to both speakers). The last chunk of a file is
 * shorter; `null` when no sample falls in the chunk.
 */
export function assembleChunk(
  buffers: readonly TimedAudioBuffer[],
  index: number,
  origin: number,
  createBuffer: AudioBufferFactory,
  previous: ChunkPlacement | null = null,
): AssembledChunk | null {
  const first = buffers[0];
  const lastBuffer = buffers.at(-1);
  if (first === undefined || lastBuffer === undefined) return null;
  const { sampleRate } = first.buffer;
  const chunkStart = Math.round(index * CHUNK_SECONDS * sampleRate);
  const positions = placements(buffers, origin, previous);
  const end = Math.max(...buffers.map((timed, at) => (positions[at] ?? 0) + timed.buffer.length));
  const length = Math.min(Math.round(CHUNK_SECONDS * sampleRate), end - chunkStart);
  if (length <= 0) return null;
  const channels = Math.max(...buffers.map((timed) => timed.buffer.numberOfChannels));
  const chunk = createBuffer(channels, length, sampleRate);
  buffers.forEach(({ buffer }, at) => {
    const offset = (positions[at] ?? 0) - chunkStart;
    const skip = Math.max(0, -offset);
    const count = Math.min(buffer.length - skip, length - Math.max(0, offset));
    if (count <= 0) return;
    for (let channel = 0; channel < channels; channel++) {
      const source = buffer.getChannelData(Math.min(channel, buffer.numberOfChannels - 1));
      chunk.copyToChannel(source.subarray(skip, skip + count), channel, Math.max(0, offset));
    }
  });
  const last = { position: positions.at(-1) ?? 0, length: lastBuffer.buffer.length };
  return { chunk: { buffer: chunk, start: index * CHUNK_SECONDS }, last };
}

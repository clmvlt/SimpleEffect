import { ALL_FORMATS, AudioBufferSink, Input, UrlSource } from 'mediabunny';

import { DecodeError, asDecodeError } from '../decode/decode-error';
import { assembleChunk, type AudioChunk, type TimedAudioBuffer } from './audio-chunk';
import { CHUNK_SECONDS } from './audio-plan';

/** Decoded chunks of one file's sound. */
export interface AudioChunkReader {
  /**
   * Chunk `index` (media seconds `[index, index + 1)`), `null` past the end. Rejects with a
   * `DecodeError` when the file has no sound this machine can decode.
   */
  chunk(index: number): Promise<AudioChunk | null>;
  dispose(): void;
}

/** Creates the reader of the file at `url`. */
export type AudioReaderFactory = (url: string) => AudioChunkReader;

/** Bytes of the file each reader keeps in memory. */
const SOURCE_CACHE_BYTES = 8 * 1024 * 1024;

interface OpenAudio {
  readonly sink: AudioBufferSink;
  /** Container time of media time 0. */
  readonly origin: number;
}

function createAudioBuffer(channels: number, length: number, sampleRate: number): AudioBuffer {
  return new AudioBuffer({ numberOfChannels: channels, length, sampleRate });
}

/**
 * An {@link AudioChunkReader} over an HTTP URL (se-core's `/media` routes) with Mediabunny:
 * demuxing in JavaScript, decoding with WebCodecs, each chunk decoded on its own. Media time 0 is
 * the first video frame when the file has video (the picture's origin, so sound stays in sync
 * with it), otherwise the first audio sample.
 */
export class MediabunnyAudioSource implements AudioChunkReader {
  private readonly input: Input;
  private track: Promise<OpenAudio> | null = null;

  constructor(url: string) {
    this.input = new Input({
      source: new UrlSource(url, { maxCacheSize: SOURCE_CACHE_BYTES }),
      formats: ALL_FORMATS,
    });
  }

  async chunk(index: number): Promise<AudioChunk | null> {
    const { sink, origin } = await this.openTrack();
    const start = origin + index * CHUNK_SECONDS;
    const buffers: TimedAudioBuffer[] = [];
    try {
      for await (const wrapped of sink.buffers(start, start + CHUNK_SECONDS)) buffers.push(wrapped);
    } catch (error: unknown) {
      throw asDecodeError(error, 'readFailed', 'Reading or decoding the sound failed.');
    }
    return assembleChunk(buffers, index, origin, createAudioBuffer);
  }

  dispose(): void {
    this.input.dispose();
  }

  private openTrack(): Promise<OpenAudio> {
    this.track ??= this.readTrack();
    return this.track;
  }

  private async readTrack(): Promise<OpenAudio> {
    try {
      const audio = await this.input.getPrimaryAudioTrack();
      if (audio === null) throw new DecodeError('noAudioTrack', 'The file has no sound.');
      if (!(await audio.canDecode())) {
        throw new DecodeError(
          'codecNotDecodable',
          'This machine cannot decode the sound codec: the asset needs a mezzanine.',
        );
      }
      const video = await this.input.getPrimaryVideoTrack();
      const first = await (video ?? audio).getFirstTimestamp();
      return { sink: new AudioBufferSink(audio), origin: Math.max(0, first) };
    } catch (error: unknown) {
      throw asDecodeError(error, 'readFailed', 'The sound of the file could not be read.');
    }
  }
}

/** The default factory: Mediabunny over the URL. */
export const createMediabunnyAudioSource: AudioReaderFactory = (url) =>
  new MediabunnyAudioSource(url);

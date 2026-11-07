import { ALL_FORMATS, Input, UrlSource, VideoSampleSink, type VideoSample } from 'mediabunny';

import type { PictureRotation } from '../composite/layout';
import { DecodeError, asDecodeError } from './decode-error';

/** One decoded video frame and the media time range it is shown for. */
export interface DecodedVideoFrame {
  /** Owned by the caller, who must close it. */
  readonly frame: VideoFrame;
  /** Seconds from the first frame of the file: `[start, end)`. */
  readonly start: number;
  readonly end: number;
  readonly rotation: PictureRotation;
  readonly mirrored: boolean;
}

/** Frame-accurate access to the picture of one file. */
export interface VideoFrameSource {
  /**
   * The frame shown at `time` (seconds from the first frame), or `null` before it. Rejects with a
   * `DecodeError` when the file cannot be read or its codec cannot be decoded here.
   */
  frameAt(time: number): Promise<DecodedVideoFrame | null>;
  /**
   * The frames from the one shown at `start` onwards, in presentation order, decoded by one
   * decoder (playback). Closing the iterator (`return()`) stops decoding; frames it yielded are
   * owned by the caller. Rejects like `frameAt`.
   */
  frames(start: number): AsyncGenerator<DecodedVideoFrame, void, undefined>;
  /** Stops reading and frees the demuxer (pending reads reject). */
  dispose(): void;
}

/** Creates the source reading the file at `url`. */
export type VideoSourceFactory = (url: string) => VideoFrameSource;

/** Bytes of the file each source keeps in memory (Mediabunny's default is 64 MiB). */
const SOURCE_CACHE_BYTES = 16 * 1024 * 1024;

interface OpenTrack {
  readonly sink: VideoSampleSink;
  /** Timestamp of the first frame: media time 0 (B-frame delays without an edit list). */
  readonly offset: number;
}

/** Copies `sample` into a `VideoFrame` the caller owns, and closes the sample. */
function decodedFrame(sample: VideoSample, offset: number): DecodedVideoFrame {
  try {
    const start = sample.timestamp - offset;
    return {
      frame: sample.toVideoFrame(),
      start,
      end: start + sample.duration,
      rotation: rotationOf(sample.rotation),
      mirrored: sample.flip,
    };
  } finally {
    sample.close();
  }
}

function rotationOf(degrees: number): PictureRotation {
  const normalised = ((degrees % 360) + 360) % 360;
  return normalised === 90 || normalised === 180 || normalised === 270 ? normalised : 0;
}

/**
 * A {@link VideoFrameSource} over an HTTP URL (se-core's `/media` routes, Range requests) with
 * Mediabunny: demuxing in JavaScript, decoding with WebCodecs. Media time 0 is the first frame's
 * timestamp (or 0 when frames before it are hidden by an edit list), as for ffmpeg, so an
 * original and its transcoded mezzanine or proxy share times.
 *
 * `frameAt` (scrubbing) configures a decoder per call and decodes from the previous key frame;
 * `frames` (playback) keeps one decoder running through the file.
 */
export class MediabunnyVideoSource implements VideoFrameSource {
  private readonly input: Input;
  private track: Promise<OpenTrack> | null = null;

  /**
   * `hardwareAcceleration` is a decoder hint: golden-frame tests ask for software decoding so
   * every machine produces the same pixels.
   */
  constructor(
    url: string,
    private readonly hardwareAcceleration: HardwareAcceleration = 'no-preference',
  ) {
    this.input = new Input({
      source: new UrlSource(url, { maxCacheSize: SOURCE_CACHE_BYTES }),
      formats: ALL_FORMATS,
    });
  }

  async frameAt(time: number): Promise<DecodedVideoFrame | null> {
    const { sink, offset } = await this.openTrack();
    let sample;
    try {
      sample = await sink.getSample(offset + time);
    } catch (error: unknown) {
      throw asDecodeError(error, 'readFailed', 'Reading or decoding the video failed.');
    }
    if (sample === null) return null;
    return decodedFrame(sample, offset);
  }

  async *frames(start: number): AsyncGenerator<DecodedVideoFrame, void, undefined> {
    const { sink, offset } = await this.openTrack();
    const samples = sink.samples(offset + start);
    try {
      for (;;) {
        let next;
        try {
          next = await samples.next();
        } catch (error: unknown) {
          throw asDecodeError(error, 'readFailed', 'Reading or decoding the video failed.');
        }
        if (next.done === true) return;
        yield decodedFrame(next.value, offset);
      }
    } finally {
      await samples.return();
    }
  }

  dispose(): void {
    this.input.dispose();
  }

  private openTrack(): Promise<OpenTrack> {
    this.track ??= this.readTrack();
    return this.track;
  }

  private async readTrack(): Promise<OpenTrack> {
    try {
      const track = await this.input.getPrimaryVideoTrack();
      if (track === null) {
        throw new DecodeError('noVideoTrack', 'The file has no video track.');
      }
      if (!(await track.canDecode())) {
        throw new DecodeError(
          'codecNotDecodable',
          'This machine cannot decode the video codec: the asset needs a mezzanine.',
        );
      }
      const offset = Math.max(0, await track.getFirstTimestamp());
      const sink = new VideoSampleSink(track, { hardwareAcceleration: this.hardwareAcceleration });
      return { sink, offset };
    } catch (error: unknown) {
      throw asDecodeError(error, 'readFailed', 'The video file could not be read.');
    }
  }
}

/** The default factory: Mediabunny over the URL. */
export const createMediabunnyVideoSource: VideoSourceFactory = (url) =>
  new MediabunnyVideoSource(url);

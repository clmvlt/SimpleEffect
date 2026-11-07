/**
 * Fake decoding for engine unit tests in Node (no WebCodecs): frames that record their closing and
 * a video source decoding a constant-rate file whose resolution the test controls.
 */
import type { DecodedVideoFrame, VideoFrameSource } from '../decode/video-source';

/** A stand-in `VideoFrame`/`ImageBitmap`: remembers whether it was closed. */
export class FakeFrame {
  closed = false;
  readonly displayWidth: number;
  readonly displayHeight: number;
  readonly width: number;
  readonly height: number;

  constructor(
    readonly index: number,
    width = 16,
    height = 9,
  ) {
    this.displayWidth = width;
    this.displayHeight = height;
    this.width = width;
    this.height = height;
  }

  close(): void {
    if (this.closed) throw new Error(`frame ${String(this.index)} closed twice`);
    this.closed = true;
  }
}

/** A pending `frameAt` call the test resolves by hand (see {@link FakeVideoSource.manual}). */
interface PendingDecode {
  readonly time: number;
  readonly resolve: () => void;
}

/** A constant-rate video of `frameCount` frames at `fps`, decoded by the test's rules. */
export class FakeVideoSource implements VideoFrameSource {
  readonly decodedTimes: number[] = [];
  readonly frames: FakeFrame[] = [];
  readonly pending: PendingDecode[] = [];
  disposed = false;
  /** When true, each decode waits until the test calls `pending[i].resolve()`. */
  manual = false;
  failure: Error | null = null;

  constructor(
    readonly url: string,
    private readonly fps = 25,
    private readonly frameCount = 250,
  ) {}

  async frameAt(time: number): Promise<DecodedVideoFrame | null> {
    if (this.failure !== null) throw this.failure;
    if (this.manual) {
      await new Promise<void>((resolve) => this.pending.push({ time, resolve }));
    }
    this.decodedTimes.push(time);
    const index = Math.min(Math.floor(time * this.fps), this.frameCount - 1);
    if (index < 0) return null;
    const frame = new FakeFrame(index);
    this.frames.push(frame);
    return {
      frame: frame as unknown as VideoFrame,
      start: index / this.fps,
      end: (index + 1) / this.fps,
      rotation: 0,
      mirrored: false,
    };
  }

  dispose(): void {
    this.disposed = true;
  }
}

/** Resolves once pending promise callbacks (microtasks and the next macrotask) have run. */
export function settle(): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, 0));
}

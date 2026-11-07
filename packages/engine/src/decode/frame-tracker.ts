/** Kinds of decoded pictures the engine holds: GPU-backed and closed explicitly. */
export type PictureKind = 'videoFrame' | 'image';

/** Something holding pixels that must be closed (`VideoFrame`, `ImageBitmap`). */
export interface Closable {
  close(): void;
}

/** Pictures currently open, by kind. */
export interface OpenPictureCounts {
  readonly videoFrames: number;
  readonly images: number;
}

/**
 * Counts the pictures the engine opened and has not closed yet: the leak check of the decoder pool
 * (a `VideoFrame` that is never closed holds GPU memory and can stall the hardware decoder).
 * Closing goes through the tracker, which makes it idempotent.
 */
export class FrameTracker {
  private readonly open = new Map<Closable, PictureKind>();

  /** Starts tracking a picture the engine now owns, and returns it. */
  adopt<T extends Closable>(picture: T, kind: PictureKind): T {
    this.open.set(picture, kind);
    return picture;
  }

  /** Closes a tracked picture (does nothing for one already closed). */
  close(picture: Closable): void {
    if (!this.open.delete(picture)) return;
    picture.close();
  }

  /** Pictures adopted and not closed yet. */
  counts(): OpenPictureCounts {
    let videoFrames = 0;
    let images = 0;
    for (const kind of this.open.values()) {
      if (kind === 'videoFrame') videoFrames += 1;
      else images += 1;
    }
    return { videoFrames, images };
  }
}

/** The tracker shared by every decoder pool of the page or Worker (the dev leak counter). */
export const pictureTracker = new FrameTracker();

/** Pictures open in this page or Worker (dev counter; must drop to zero once pools are destroyed). */
export function openPictureCounts(): OpenPictureCounts {
  return pictureTracker.counts();
}

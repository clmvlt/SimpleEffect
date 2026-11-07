/**
 * Decoding: the decoder pool (Mediabunny + WebCodecs), its picture cache and leak counter, and the
 * choice of the decoded file (original, mezzanine, proxy).
 */
export { DecodeError, type DecodeFailure } from './decode-error';
export {
  DecoderPool,
  type DecoderPoolOptions,
  type DecoderPoolStats,
  type MediaFileRef,
} from './decoder-pool';
export type { PictureLease } from './frame-cache';
export {
  FrameTracker,
  openPictureCounts,
  pictureTracker,
  type OpenPictureCounts,
  type PictureKind,
} from './frame-tracker';
export { fetchImageBitmap, type ImageLoader } from './image-loader';
export {
  chooseVariant,
  type MediaLocator,
  type MediaVariant,
  type VariantChoice,
  type VariantOptions,
} from './media-variant';
export {
  MediabunnyVideoSource,
  createMediabunnyVideoSource,
  type DecodedVideoFrame,
  type VideoFrameSource,
  type VideoSourceFactory,
} from './video-source';

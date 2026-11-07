import { DecodeError } from './decode-error';

/** Loads the still image at `url` as an upright `ImageBitmap` owned by the caller. */
export type ImageLoader = (url: string) => Promise<ImageBitmap>;

/**
 * The default loader: fetch, then `createImageBitmap` with the file's EXIF orientation applied
 * (works in the page and in Workers).
 */
export const fetchImageBitmap: ImageLoader = async (url) => {
  let blob: Blob;
  try {
    const response = await fetch(url);
    if (!response.ok) {
      throw new Error(`HTTP ${String(response.status)}`);
    }
    blob = await response.blob();
  } catch (error: unknown) {
    throw new DecodeError('readFailed', 'The image file could not be read.', { cause: error });
  }
  try {
    return await createImageBitmap(blob, { imageOrientation: 'from-image' });
  } catch (error: unknown) {
    throw new DecodeError('imageNotDecodable', 'This image format cannot be decoded here.', {
      cause: error,
    });
  }
};

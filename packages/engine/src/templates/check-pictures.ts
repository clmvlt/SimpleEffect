/** JPEG quality of check pictures (the agent judges layout and timing, not fine detail). */
const CHECK_JPEG_QUALITY = 0.8;

/** Pixels of `bitmap` as RGBA bytes. */
function pixelsOf(bitmap: ImageBitmap): Uint8ClampedArray {
  const canvas = new OffscreenCanvas(bitmap.width, bitmap.height);
  const context = canvas.getContext('2d');
  if (context === null) return new Uint8ClampedArray();
  context.drawImage(bitmap, 0, 0);
  return context.getImageData(0, 0, bitmap.width, bitmap.height).data;
}

/** How many pixels differ between two renders (every pixel when the sizes differ). */
export function differingPixels(first: ImageBitmap, second: ImageBitmap): number {
  if (first.width !== second.width || first.height !== second.height) {
    return Math.max(first.width * first.height, second.width * second.height);
  }
  const a = pixelsOf(first);
  const b = pixelsOf(second);
  let differing = 0;
  for (let index = 0; index < a.length; index += 4) {
    if (
      a[index] !== b[index] ||
      a[index + 1] !== b[index + 1] ||
      a[index + 2] !== b[index + 2] ||
      a[index + 3] !== b[index + 3]
    ) {
      differing += 1;
    }
  }
  return differing;
}

/**
 * `bitmap` scaled to `width` × `height` over black (how the viewer shows a template over an empty
 * frame), as a base64 JPEG; a black picture when there is no bitmap.
 */
export async function encodeJpegBase64(
  bitmap: ImageBitmap | null,
  width: number,
  height: number,
): Promise<string> {
  const canvas = new OffscreenCanvas(width, height);
  const context = canvas.getContext('2d');
  if (context !== null) {
    context.fillStyle = 'black';
    context.fillRect(0, 0, width, height);
    if (bitmap !== null) context.drawImage(bitmap, 0, 0, width, height);
  }
  const blob = await canvas.convertToBlob({ type: 'image/jpeg', quality: CHECK_JPEG_QUALITY });
  return base64Of(new Uint8Array(await blob.arrayBuffer()));
}

/** Base64 of bytes, in chunks (`String.fromCharCode` takes a bounded number of arguments). */
export function base64Of(bytes: Uint8Array): string {
  let binary = '';
  const chunk = 0x8000;
  for (let start = 0; start < bytes.length; start += chunk) {
    binary += String.fromCharCode(...bytes.subarray(start, start + chunk));
  }
  return btoa(binary);
}

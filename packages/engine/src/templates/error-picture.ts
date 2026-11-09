/** Stripes of the error picture, in output pixels. */
const STRIPE = 24;

/**
 * The picture a failing template shows in its place (prompt 24): a red frame with diagonal
 * stripes over transparency — visible in the viewer and in frames for agents, never mistaken for
 * content. No text: the engine has no translations; the inspector shows the message.
 */
export async function errorPicture(width: number, height: number): Promise<ImageBitmap> {
  const canvas = new OffscreenCanvas(Math.max(1, width), Math.max(1, height));
  const context = canvas.getContext('2d');
  if (context === null) return createImageBitmap(canvas);
  context.fillStyle = 'rgba(220, 38, 38, 0.18)';
  context.fillRect(0, 0, width, height);
  context.strokeStyle = 'rgba(220, 38, 38, 0.55)';
  context.lineWidth = STRIPE / 3;
  context.beginPath();
  for (let offset = -height; offset < width; offset += STRIPE) {
    context.moveTo(offset, height);
    context.lineTo(offset + height, 0);
  }
  context.stroke();
  context.strokeStyle = 'rgba(220, 38, 38, 0.9)';
  context.lineWidth = Math.max(4, Math.round(Math.min(width, height) / 90));
  context.strokeRect(0, 0, width, height);
  return canvas.transferToImageBitmap();
}

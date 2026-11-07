import type { ViewerZoom } from '../store/viewer-store';

/** A width and a height. */
export interface Size {
  readonly width: number;
  readonly height: number;
}

/** Where the picture sits in the viewer and how many pixels the engine draws. */
export interface ViewerLayout {
  /** Displayed size of the picture in CSS pixels (the sequence ratio, always). */
  readonly display: Size;
  /** Size of the canvas backing store in device pixels, never above the sequence size. */
  readonly pixels: Size;
  /** Displayed size divided by the sequence size (1 = one sequence pixel per CSS pixel). */
  readonly scale: number;
}

/** Room kept around a fitted picture so it never touches the panel edges. */
export const FIT_MARGIN = 8;

/**
 * Lays out the picture of a `frame`-sized sequence in a `stage` at `zoom`: `fit` scales it into
 * the stage (minus a margin), fixed zooms show `zoom` CSS pixels per sequence pixel (the stage
 * scrolls when larger). The engine draws at most the sequence's own resolution: more pixels would
 * only upscale the video, and the browser scales the canvas the rest of the way.
 */
export function viewerLayout(
  stage: Size,
  frame: Size,
  zoom: ViewerZoom,
  devicePixelRatio: number,
): ViewerLayout {
  const scale =
    zoom === 'fit'
      ? Math.max(
          0,
          Math.min(
            (stage.width - 2 * FIT_MARGIN) / frame.width,
            (stage.height - 2 * FIT_MARGIN) / frame.height,
          ),
        )
      : zoom;
  const display = { width: frame.width * scale, height: frame.height * scale };
  const ratio = devicePixelRatio > 0 ? devicePixelRatio : 1;
  const pixels = {
    width: Math.max(1, Math.min(frame.width, Math.round(display.width * ratio))),
    height: Math.max(1, Math.min(frame.height, Math.round(display.height * ratio))),
  };
  return { display, pixels, scale };
}

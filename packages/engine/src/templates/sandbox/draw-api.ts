import type { TemplateHelpers } from './helpers';
import type { TextApi } from './text-api';

/** What the drawing API is built on for one render. */
export interface DrawApiOptions {
  /** The real context, already scaled from design pixels to output pixels (`scale`). */
  readonly real: OffscreenCanvasRenderingContext2D;
  /** Output pixels per design pixel. */
  readonly scale: number;
  /** The design frame and the instance's length (seconds). */
  readonly width: number;
  readonly height: number;
  readonly duration: number;
  /** Pictures by handle id. */
  readonly images: ReadonlyMap<string, ImageBitmap>;
  /** Text helpers (`createTextApi`) over the same context. */
  readonly textApi: TextApi;
  readonly helpers: TemplateHelpers;
}

/**
 * The `ctx` a template draws with: a restricted Canvas 2D (shapes, paths, gradients, text with a
 * multi-line layout helper, images of `asset` parameters, transforms, `globalAlpha`, blend modes,
 * shadows, blur) in **design pixels** (shorter side 1080), plus the helpers. It never exposes the
 * canvas or the real context. Shadow sizes and blur are scaled to output pixels (the canvas does
 * not transform them).
 *
 * **Self-contained**: serialized with `toString()` into the Worker's source (`sandbox-source.ts`).
 * It may use only its parameters and JavaScript built-ins — no import, no outer variable.
 */
export function createDrawApi(options: DrawApiOptions): object {
  const { real, scale, images, textApi, helpers } = options;
  const blends = new Set([
    'source-over',
    'source-atop',
    'destination-over',
    'destination-out',
    'lighter',
    'multiply',
    'screen',
    'overlay',
    'darken',
    'lighten',
    'color-dodge',
    'color-burn',
    'hard-light',
    'soft-light',
    'difference',
    'exclusion',
    'hue',
    'saturation',
    'color',
    'luminosity',
    'xor',
  ]);
  const gradients = new WeakMap<object, CanvasGradient>();
  const paths = new WeakMap<object, Path2D>();
  const style = (value: unknown) => {
    if (typeof value === 'string') return value;
    const gradient = typeof value === 'object' && value !== null ? gradients.get(value) : undefined;
    if (gradient === undefined) throw new TypeError('use a colour string or a gradient of ctx');
    return gradient;
  };
  const gradient = (real: CanvasGradient) => {
    const handle = Object.freeze({
      addColorStop: (offset: number, color: string) => {
        real.addColorStop(offset, color);
        return handle;
      },
    });
    gradients.set(handle, real);
    return handle;
  };
  const path = (value: unknown) =>
    typeof value === 'object' && value !== null ? paths.get(value) : undefined;
  const image = (handle: unknown) => {
    const id =
      typeof handle === 'object' && handle !== null && 'id' in handle ? String(handle.id) : '';
    const bitmap = images.get(id);
    if (bitmap === undefined) {
      throw new TypeError(
        'drawImage takes the image of an asset parameter (params.<name>), not null',
      );
    }
    return bitmap;
  };
  const api: Record<string, unknown> = {
    save: () => {
      real.save();
    },
    restore: () => {
      real.restore();
    },
    translate: (x: number, y: number) => {
      real.translate(x, y);
    },
    rotate: (angle: number) => {
      real.rotate(angle);
    },
    scale: (x: number, y?: number) => {
      real.scale(x, y ?? x);
    },
    transform: (a: number, b: number, c: number, d: number, e: number, f: number) => {
      real.transform(a, b, c, d, e, f);
    },
    setTransform: (a: number, b: number, c: number, d: number, e: number, f: number) => {
      real.setTransform(scale, 0, 0, scale, 0, 0);
      real.transform(a, b, c, d, e, f);
    },
    resetTransform: () => {
      real.setTransform(scale, 0, 0, scale, 0, 0);
    },
    beginPath: () => {
      real.beginPath();
    },
    closePath: () => {
      real.closePath();
    },
    moveTo: (x: number, y: number) => {
      real.moveTo(x, y);
    },
    lineTo: (x: number, y: number) => {
      real.lineTo(x, y);
    },
    bezierCurveTo: (a: number, b: number, c: number, d: number, x: number, y: number) => {
      real.bezierCurveTo(a, b, c, d, x, y);
    },
    quadraticCurveTo: (a: number, b: number, x: number, y: number) => {
      real.quadraticCurveTo(a, b, x, y);
    },
    arc: (x: number, y: number, r: number, from: number, to: number, ccw?: boolean) => {
      real.arc(x, y, r, from, to, ccw);
    },
    arcTo: (a: number, b: number, c: number, d: number, r: number) => {
      real.arcTo(a, b, c, d, r);
    },
    ellipse: (
      x: number,
      y: number,
      rx: number,
      ry: number,
      rotation: number,
      from: number,
      to: number,
      ccw?: boolean,
    ) => {
      real.ellipse(x, y, rx, ry, rotation, from, to, ccw);
    },
    rect: (x: number, y: number, w: number, h: number) => {
      real.rect(x, y, w, h);
    },
    roundRect: (x: number, y: number, w: number, h: number, radii?: number | number[]) => {
      real.roundRect(x, y, w, h, radii);
    },
    path: (data: unknown) => {
      const handle = Object.freeze({});
      paths.set(handle, new Path2D(String(data)));
      return handle;
    },
    fill: (first?: unknown, rule?: CanvasFillRule) => {
      const shape = path(first);
      if (shape !== undefined) real.fill(shape, rule);
      else real.fill(typeof first === 'string' ? (first as CanvasFillRule) : undefined);
    },
    stroke: (shape?: unknown) => {
      const found = path(shape);
      if (found === undefined) real.stroke();
      else real.stroke(found);
    },
    clip: (first?: unknown, rule?: CanvasFillRule) => {
      const shape = path(first);
      if (shape !== undefined) real.clip(shape, rule);
      else real.clip(typeof first === 'string' ? (first as CanvasFillRule) : undefined);
    },
    fillRect: (x: number, y: number, w: number, h: number) => {
      real.fillRect(x, y, w, h);
    },
    strokeRect: (x: number, y: number, w: number, h: number) => {
      real.strokeRect(x, y, w, h);
    },
    clearRect: (x: number, y: number, w: number, h: number) => {
      real.clearRect(x, y, w, h);
    },
    fillText: (text: unknown, x: number, y: number, maxWidth?: number) => {
      real.fillText(String(text), x, y, maxWidth);
    },
    strokeText: (text: unknown, x: number, y: number, maxWidth?: number) => {
      real.strokeText(String(text), x, y, maxWidth);
    },
    measureText: (text: unknown) => {
      const metrics = real.measureText(String(text));
      return Object.freeze({
        width: metrics.width,
        actualBoundingBoxLeft: metrics.actualBoundingBoxLeft,
        actualBoundingBoxRight: metrics.actualBoundingBoxRight,
        actualBoundingBoxAscent: metrics.actualBoundingBoxAscent,
        actualBoundingBoxDescent: metrics.actualBoundingBoxDescent,
        fontBoundingBoxAscent: metrics.fontBoundingBoxAscent,
        fontBoundingBoxDescent: metrics.fontBoundingBoxDescent,
      });
    },
    setLineDash: (segments: number[]) => {
      real.setLineDash(segments);
    },
    getLineDash: () => real.getLineDash(),
    createLinearGradient: (a: number, b: number, c: number, d: number) =>
      gradient(real.createLinearGradient(a, b, c, d)),
    createRadialGradient: (a: number, b: number, r1: number, c: number, d: number, r2: number) =>
      gradient(real.createRadialGradient(a, b, r1, c, d, r2)),
    createConicGradient: (angle: number, x: number, y: number) =>
      gradient(real.createConicGradient(angle, x, y)),
    drawImage: (handle: unknown, ...rest: number[]) => {
      const bitmap = image(handle);
      if (rest.length === 2) real.drawImage(bitmap, rest[0] ?? 0, rest[1] ?? 0);
      else if (rest.length === 4)
        real.drawImage(bitmap, rest[0] ?? 0, rest[1] ?? 0, rest[2] ?? 0, rest[3] ?? 0);
      else if (rest.length === 8) {
        const [sx = 0, sy = 0, sw = 0, sh = 0, dx = 0, dy = 0, dw = 0, dh = 0] = rest;
        real.drawImage(bitmap, sx, sy, sw, sh, dx, dy, dw, dh);
      } else
        throw new TypeError('drawImage(image, x, y), (image, x, y, w, h) or the 9-argument form');
    },
    setFont: textApi.setFont,
    layoutText: textApi.layoutText,
    text: textApi.text,
    ...helpers,
  };
  const property = (name: string, read: () => unknown, write?: (value: never) => void) => {
    Object.defineProperty(api, name, {
      get: read,
      set:
        write ??
        (() => {
          throw new TypeError(`ctx.${name} is read-only`);
        }),
      enumerable: true,
    });
  };
  let fill: unknown = '#000000';
  let strokeColor: unknown = '#000000';
  let blur = 0;
  property('width', () => options.width);
  property('height', () => options.height);
  property('duration', () => options.duration);
  property(
    'fillStyle',
    () => fill,
    (value: unknown) => {
      real.fillStyle = style(value);
      fill = value;
    },
  );
  property(
    'strokeStyle',
    () => strokeColor,
    (value: unknown) => {
      real.strokeStyle = style(value);
      strokeColor = value;
    },
  );
  property(
    'font',
    () => real.font,
    (value: unknown) => {
      real.font = String(value);
      textApi.noteFamilies(String(value));
    },
  );
  property(
    'filter',
    () => (blur > 0 ? `blur(${String(blur)}px)` : 'none'),
    (value: unknown) => {
      const radius = /^\s*blur\(\s*(\d*\.?\d+)px\s*\)\s*$/.exec(String(value))?.[1];
      blur = radius === undefined ? 0 : Number(radius);
      real.filter = blur > 0 ? `blur(${String(blur * scale)}px)` : 'none';
    },
  );
  property(
    'globalCompositeOperation',
    () => real.globalCompositeOperation,
    (value: unknown) => {
      if (typeof value !== 'string' || !blends.has(value)) {
        throw new TypeError(`'${String(value)}' is not a blend mode`);
      }
      real.globalCompositeOperation = value as GlobalCompositeOperation;
    },
  );
  for (const name of ['shadowBlur', 'shadowOffsetX', 'shadowOffsetY'] as const) {
    property(
      name,
      () => real[name] / scale,
      (value: unknown) => {
        real[name] = Number(value) * scale;
      },
    );
  }
  for (const name of [
    'lineWidth',
    'lineCap',
    'lineJoin',
    'miterLimit',
    'lineDashOffset',
    'globalAlpha',
    'textAlign',
    'textBaseline',
    'letterSpacing',
    'shadowColor',
    'imageSmoothingEnabled',
  ] as const) {
    property(
      name,
      () => real[name],
      (value: never) => {
        (real as unknown as Record<string, unknown>)[name] = value;
      },
    );
  }
  return Object.freeze(api);
}

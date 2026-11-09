/** What the text helpers work on for one render. */
export interface TextApiOptions {
  /** The real context (design pixels already scaled to output pixels). */
  readonly real: OffscreenCanvasRenderingContext2D;
  /** Font families the Worker has faces for, lower case. */
  readonly loadedFamilies: ReadonlySet<string>;
  /** Filled with the families asked for that the Worker lacks, lower case. */
  readonly missingFamilies: Set<string>;
}

/** One laid-out line. */
export interface LaidOutLine {
  readonly text: string;
  readonly width: number;
}

/** What `layoutText` and `text` answer. */
export interface TextLayout {
  readonly lines: readonly LaidOutLine[];
  readonly width: number;
  readonly height: number;
}

/** The text part of `ctx`. */
export interface TextApi {
  /** Records the families of a CSS font the Worker lacks (they are sent, then the frame redrawn). */
  readonly noteFamilies: (font: string) => void;
  readonly setFont: (spec: Record<string, unknown>) => void;
  readonly layoutText: (text: unknown, spec?: Record<string, unknown>) => TextLayout;
  readonly text: (
    text: unknown,
    x: number,
    y: number,
    spec?: Record<string, unknown>,
  ) => TextLayout;
}

/**
 * The text helpers of `ctx`: `setFont({ family, size, weight, style })`, `layoutText(text, spec)`
 * (lines wrapped at `maxWidth`, measured) and `text(text, x, y, spec)` (drawn at an anchor:
 * `align` left/center/right, `valign` top/middle/bottom, `color`, `stroke: { color, width }`,
 * `lineHeight` as a multiple of the size, `letterSpacing`). Helpers leave the context's state as
 * they found it. Families the Worker lacks are noted so the runtime sends them.
 *
 * **Self-contained**: serialized with `toString()` into the Worker's source (`sandbox-source.ts`).
 * It may use only its parameters and JavaScript built-ins — no import, no outer variable.
 */
export function createTextApi(options: TextApiOptions): TextApi {
  const { real, loadedFamilies, missingFamilies } = options;
  const generic = new Set(['serif', 'sans-serif', 'monospace', 'cursive', 'fantasy', 'system-ui']);
  const noteFamilies = (font: string) => {
    const families = /\d*\.?\d+(?:px|pt|em|rem|%)(?:\s*\/\s*\S+)?\s+(.+)$/.exec(font)?.[1] ?? '';
    for (const raw of families.split(',')) {
      const family = raw
        .trim()
        .replace(/^["']|["']$/g, '')
        .toLowerCase();
      if (family !== '' && !generic.has(family) && !loadedFamilies.has(family)) {
        missingFamilies.add(family);
      }
    }
  };
  const fontOf = (spec: Record<string, unknown>) => {
    const family = typeof spec['family'] === 'string' ? spec['family'] : 'Inter';
    const size = typeof spec['size'] === 'number' ? spec['size'] : 48;
    const declared = spec['weight'];
    const weight = typeof declared === 'number' || typeof declared === 'string' ? declared : 400;
    const style = spec['style'] === 'italic' ? 'italic' : 'normal';
    return `${style} ${String(weight)} ${String(size)}px "${family.replace(/"/g, '')}", Inter, sans-serif`;
  };
  const setFont = (spec: Record<string, unknown>) => {
    const font = fontOf(spec);
    real.font = font;
    noteFamilies(font);
  };
  const wrap = (text: unknown, maxWidth: number | undefined) => {
    const result: string[] = [];
    for (const paragraph of String(text).split('\n')) {
      let line = '';
      for (const word of paragraph.split(/\s+/).filter((part) => part !== '')) {
        const candidate = line === '' ? word : `${line} ${word}`;
        if (maxWidth !== undefined && line !== '' && real.measureText(candidate).width > maxWidth) {
          result.push(line);
          line = word;
        } else line = candidate;
      }
      result.push(line);
    }
    return result;
  };
  /** Lays the text out with the spec's font (the caller saves and restores the context). */
  const layout = (text: unknown, spec: Record<string, unknown>) => {
    setFont(spec);
    if (typeof spec['letterSpacing'] === 'number') {
      real.letterSpacing = `${String(spec['letterSpacing'])}px`;
    }
    const size = typeof spec['size'] === 'number' ? spec['size'] : 48;
    const lineHeight = (typeof spec['lineHeight'] === 'number' ? spec['lineHeight'] : 1.2) * size;
    const maxWidth = typeof spec['maxWidth'] === 'number' ? spec['maxWidth'] : undefined;
    const lines = wrap(text, maxWidth).map((line) => ({
      text: line,
      width: real.measureText(line).width,
    }));
    const width = Math.max(0, ...lines.map((line) => line.width));
    return { lines, width, height: lines.length * lineHeight, lineHeight };
  };
  const draw = (
    measured: ReturnType<typeof layout>,
    x: number,
    y: number,
    spec: Record<string, unknown>,
  ) => {
    const align = spec['align'] === 'center' || spec['align'] === 'right' ? spec['align'] : 'left';
    const valign =
      spec['valign'] === 'middle' || spec['valign'] === 'bottom' ? spec['valign'] : 'top';
    const shift = valign === 'middle' ? 0.5 : valign === 'bottom' ? 1 : 0;
    const top = y - measured.height * shift;
    real.textAlign = align;
    real.textBaseline = 'middle';
    if (typeof spec['color'] === 'string') real.fillStyle = spec['color'];
    const outline = spec['stroke'];
    measured.lines.forEach((line, index) => {
      const lineY = top + measured.lineHeight * (index + 0.5);
      if (typeof outline === 'object' && outline !== null) {
        const { color, width } = outline as { color?: unknown; width?: unknown };
        real.strokeStyle = typeof color === 'string' ? color : '#000000';
        real.lineWidth = (typeof width === 'number' ? width : 4) * 2;
        real.lineJoin = 'round';
        real.strokeText(line.text, x, lineY);
      }
      real.fillText(line.text, x, lineY);
    });
  };
  const result = (measured: ReturnType<typeof layout>) =>
    Object.freeze({ lines: measured.lines, width: measured.width, height: measured.height });
  return Object.freeze({
    noteFamilies,
    setFont,
    layoutText: (text: unknown, spec: Record<string, unknown> = {}) => {
      real.save();
      try {
        return result(layout(text, spec));
      } finally {
        real.restore();
      }
    },
    text: (text: unknown, x: number, y: number, spec: Record<string, unknown> = {}) => {
      real.save();
      try {
        const measured = layout(text, spec);
        draw(measured, x, y, spec);
        return result(measured);
      } finally {
        real.restore();
      }
    },
  });
}

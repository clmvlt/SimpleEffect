/** The helpers `ctx` gives templates besides drawing (documented in the agents' instructions). */
export interface TemplateHelpers {
  readonly ease: Readonly<Record<string, (t: number) => number>>;
  lerp(from: number, to: number, t: number): number;
  clamp(value: number, min: number, max: number): number;
  progress(time: number, start: number, end: number): number;
  random(seed: number | string): () => number;
  mixColor(from: string, to: string, t: number): string;
  withAlpha(color: string, alpha: number): string;
}

/**
 * Pure helpers for templates: easings (`ease.outCubic(t)`…), `lerp`, `clamp`, `progress`, a
 * seeded `random`, colour mixing. Deterministic: no clock, no global randomness.
 *
 * **Self-contained**: serialized with `toString()` into the Worker's source (`sandbox-source.ts`).
 * It may use only JavaScript built-ins — no import, no outer variable.
 */
export function createHelpers(): TemplateHelpers {
  const clamp = (value: number, min: number, max: number) => Math.min(max, Math.max(min, value));
  const unit = (t: number) => clamp(Number.isFinite(t) ? t : 0, 0, 1);
  const back = 1.70158;
  const bounce = (t: number) => {
    const n = 7.5625;
    const d = 2.75;
    if (t < 1 / d) return n * t * t;
    if (t < 2 / d) return n * (t - 1.5 / d) * (t - 1.5 / d) + 0.75;
    if (t < 2.5 / d) return n * (t - 2.25 / d) * (t - 2.25 / d) + 0.9375;
    return n * (t - 2.625 / d) * (t - 2.625 / d) + 0.984375;
  };
  const curves: Record<string, (t: number) => number> = {
    linear: (t) => t,
    inQuad: (t) => t * t,
    outQuad: (t) => 1 - (1 - t) * (1 - t),
    inOutQuad: (t) => (t < 0.5 ? 2 * t * t : 1 - (-2 * t + 2) ** 2 / 2),
    inCubic: (t) => t ** 3,
    outCubic: (t) => 1 - (1 - t) ** 3,
    inOutCubic: (t) => (t < 0.5 ? 4 * t ** 3 : 1 - (-2 * t + 2) ** 3 / 2),
    inQuart: (t) => t ** 4,
    outQuart: (t) => 1 - (1 - t) ** 4,
    inOutQuart: (t) => (t < 0.5 ? 8 * t ** 4 : 1 - (-2 * t + 2) ** 4 / 2),
    inExpo: (t) => (t === 0 ? 0 : 2 ** (10 * t - 10)),
    outExpo: (t) => (t === 1 ? 1 : 1 - 2 ** (-10 * t)),
    inOutExpo: (t) =>
      t === 0 || t === 1 ? t : t < 0.5 ? 2 ** (20 * t - 10) / 2 : (2 - 2 ** (-20 * t + 10)) / 2,
    inSine: (t) => 1 - Math.cos((t * Math.PI) / 2),
    outSine: (t) => Math.sin((t * Math.PI) / 2),
    inOutSine: (t) => -(Math.cos(Math.PI * t) - 1) / 2,
    inBack: (t) => (back + 1) * t ** 3 - back * t * t,
    outBack: (t) => 1 + (back + 1) * (t - 1) ** 3 + back * (t - 1) ** 2,
    inOutBack: (t) => {
      const c = back * 1.525;
      return t < 0.5
        ? ((2 * t) ** 2 * ((c + 1) * 2 * t - c)) / 2
        : ((2 * t - 2) ** 2 * ((c + 1) * (t * 2 - 2) + c) + 2) / 2;
    },
    outElastic: (t) =>
      t === 0 || t === 1 ? t : 2 ** (-10 * t) * Math.sin(((t * 10 - 0.75) * 2 * Math.PI) / 3) + 1,
    outBounce: bounce,
    inBounce: (t) => 1 - bounce(1 - t),
  };
  const ease: Record<string, (t: number) => number> = {};
  for (const [name, curve] of Object.entries(curves)) ease[name] = (t) => curve(unit(t));
  const channels = (color: string): number[] => {
    const hex = color.trim().replace(/^#/, '');
    const full =
      hex.length === 3 || hex.length === 4
        ? hex
            .split('')
            .map((digit) => digit + digit)
            .join('')
        : hex;
    if (!/^[0-9a-fA-F]{6}([0-9a-fA-F]{2})?$/.test(full)) {
      throw new TypeError(`'${color}' is not a colour: write it #rrggbb or #rrggbbaa`);
    }
    const values = [0, 2, 4, 6].map((at) => parseInt(full.slice(at, at + 2) || 'ff', 16));
    return values;
  };
  const hex = (values: readonly number[]) =>
    '#' +
    values
      .map((value) =>
        Math.round(clamp(value, 0, 255))
          .toString(16)
          .padStart(2, '0'),
      )
      .join('');
  return Object.freeze({
    ease: Object.freeze(ease),
    lerp: (from: number, to: number, t: number) => from + (to - from) * t,
    clamp,
    progress: (time: number, start: number, end: number) =>
      end === start ? (time >= end ? 1 : 0) : unit((time - start) / (end - start)),
    random: (seed: number | string) => {
      let state = 0x9e3779b9;
      for (const character of String(seed)) {
        state = Math.imul(state ^ character.charCodeAt(0), 0x85ebca6b);
        state ^= state >>> 13;
      }
      return () => {
        state = (state + 0x6d2b79f5) | 0;
        let mixed = Math.imul(state ^ (state >>> 15), 1 | state);
        mixed = (mixed + Math.imul(mixed ^ (mixed >>> 7), 61 | mixed)) ^ mixed;
        return ((mixed ^ (mixed >>> 14)) >>> 0) / 4294967296;
      };
    },
    mixColor: (from: string, to: string, t: number) => {
      const a = channels(from);
      const b = channels(to);
      return hex(a.map((value, index) => value + ((b[index] ?? value) - value) * unit(t)));
    },
    withAlpha: (color: string, alpha: number) => {
      const values = channels(color);
      return hex([values[0] ?? 0, values[1] ?? 0, values[2] ?? 0, unit(alpha) * 255]);
    },
  });
}

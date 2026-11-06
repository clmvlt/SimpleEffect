import colorsCss from '../../tokens/colors.css?raw';
import {
  compositeOver,
  contrastRatio,
  oklchToSrgb,
  parseOklch,
  toHex,
} from '../../tokens/color-math';
import { parseTokenDeclarations, readTokens } from '../../tokens/token-css';

/** Families shown on the Tokens page, in order; a token joins the first family whose test matches. */
const FAMILIES: readonly { title: string; matches: (name: string) => boolean }[] = [
  { title: 'Text', matches: (name) => name.startsWith('text') },
  { title: 'Accent', matches: (name) => name.startsWith('accent') || name === 'focus-ring' },
  { title: 'Semantic', matches: (name) => /^(success|warning|danger|info)/.test(name) },
  { title: 'Timeline items and AI', matches: (name) => /^(item-|ai)/.test(name) },
  { title: 'Surfaces', matches: () => true },
];

const colourTokens = parseTokenDeclarations(colorsCss)
  .filter(({ name }) => name.startsWith('color-'))
  .map(({ name, value }) => ({ name: name.slice('color-'.length), value }));

const background = oklchToSrgb(
  parseOklch(readTokens(colorsCss).get('color-bg') ?? '') ?? {
    lightness: 0,
    chroma: 0,
    hue: 0,
    alpha: 1,
  },
);

/** Hex value and contrast against the app background, for a token's raw OKLCH value. */
function describeValue(value: string): { hex: string; contrast: string } | null {
  const colour = parseOklch(value);
  if (colour === null) {
    return null;
  }
  const painted = colour.alpha < 1 ? compositeOver(colour, background) : oklchToSrgb(colour);
  return { hex: toHex(painted), contrast: contrastRatio(painted, background).toFixed(2) };
}

function Swatch({ name, value }: { name: string; value: string }) {
  const details = describeValue(value);
  return (
    <li className="flex items-center gap-3 rounded-md border border-border bg-panel p-2">
      <span
        aria-hidden="true"
        className="size-10 shrink-0 rounded-sm border border-border"
        style={{ background: `var(--color-${name})` }}
      />
      <span className="flex min-w-0 flex-col">
        <code className="timecode text-xs text-text">{name}</code>
        <span className="truncate text-2xs text-text-muted">{value}</span>
        {details !== null && (
          <span className="text-2xs text-text-subtle">
            {details.hex} · {details.contrast}:1 on bg
          </span>
        )}
      </span>
    </li>
  );
}

/** Tokens grouped by family: each token goes to the first family whose test matches its name. */
const groups = FAMILIES.map(({ title }, index) => ({
  title,
  members: colourTokens.filter(
    ({ name }) => FAMILIES.findIndex((family) => family.matches(name)) === index,
  ),
}));

/**
 * Every colour token of `colors.css`, grouped by family, with its OKLCH value, the painted hex value
 * and its contrast against `bg`. Generated from the stylesheet: it cannot drift from the tokens.
 */
export function ColourSwatches() {
  return (
    <div className="flex w-[56rem] max-w-full flex-col gap-6">
      {groups.map(({ title, members }) => (
        <section key={title} className="flex flex-col gap-2">
          <h2 className="text-md font-semibold text-text">{title}</h2>
          <ul className="grid grid-cols-3 gap-2">
            {members.map((token) => (
              <Swatch key={token.name} {...token} />
            ))}
          </ul>
        </section>
      ))}
    </div>
  );
}

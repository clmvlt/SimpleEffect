/**
 * Reads design tokens straight from the token stylesheets, so tests and the Storybook Tokens page
 * always reflect the CSS (the single source of truth) without a hand-maintained copy.
 */

/** A CSS custom property declared in a token stylesheet. */
export interface TokenDeclaration {
  /** Name without the leading dashes, e.g. `color-text-muted`. */
  readonly name: string;
  /** Raw value as written, e.g. `oklch(0.77 0.01 265)`. */
  readonly value: string;
}

const DECLARATION_PATTERN = /--([\w-]+)\s*:\s*([^;]+);/g;
const COMMENT_PATTERN = /\/\*[\s\S]*?\*\//g;

/**
 * Lists the custom properties declared in `css`, in order. Namespace resets such as
 * `--color-*: initial` are skipped (the `*` makes them invalid names).
 */
export function parseTokenDeclarations(css: string): TokenDeclaration[] {
  const withoutComments = css.replace(COMMENT_PATTERN, '');
  return Array.from(withoutComments.matchAll(DECLARATION_PATTERN), ([, name = '', value = '']) => ({
    name,
    value: value.trim(),
  }));
}

/** Like {@link parseTokenDeclarations}, as a name → value map (the last declaration wins). */
export function readTokens(css: string): ReadonlyMap<string, string> {
  return new Map(parseTokenDeclarations(css).map(({ name, value }) => [name, value]));
}

/**
 * Splits `accents.css` into its `[data-accent='…']` blocks: accent name → its token map.
 */
export function readAccentVariants(css: string): ReadonlyMap<string, ReadonlyMap<string, string>> {
  const blocks = css
    .replace(COMMENT_PATTERN, '')
    .matchAll(/\[data-accent='([\w-]+)'\]\s*\{([^}]*)\}/g);
  return new Map(Array.from(blocks, ([, accent = '', body = '']) => [accent, readTokens(body)]));
}

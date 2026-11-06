import { parseTokenDeclarations } from '../../tokens/token-css';
import typographyCss from '../../tokens/typography.css?raw';

const declarations = parseTokenDeclarations(typographyCss);

const fontSizes = declarations
  .filter(({ name }) => name.startsWith('text-') && !name.includes('--'))
  .map(({ name, value }) => ({ utility: name, value }));

const fontWeights = declarations
  .filter(({ name }) => name.startsWith('font-weight-'))
  .map(({ name, value }) => ({ utility: name.replace('font-weight-', 'font-'), value }));

/** Converts a rem value of the token file to pixels (root font size 16 px). */
function toPixels(value: string): string {
  return value.endsWith('rem') ? `${String(Number.parseFloat(value) * 16)} px` : value;
}

/**
 * The type scale, weights and both families, generated from `typography.css`.
 * Utilities are applied through inline `var()` so every declared size is shown, even unused ones.
 */
export function TypeScale() {
  return (
    <div className="flex w-[48rem] max-w-full flex-col gap-6">
      <section className="flex flex-col gap-2">
        <h2 className="text-md font-semibold">Sizes</h2>
        {fontSizes.map(({ utility, value }) => (
          <div key={utility} className="flex items-baseline gap-4 border-b border-border pb-2">
            <code className="timecode w-24 shrink-0 text-xs text-text-muted">{utility}</code>
            <span className="w-14 shrink-0 text-2xs text-text-subtle">{toPixels(value)}</span>
            <span style={{ fontSize: `var(--${utility})` }}>Montage de la séquence principale</span>
          </div>
        ))}
      </section>
      <section className="flex flex-col gap-2">
        <h2 className="text-md font-semibold">Weights</h2>
        {fontWeights.map(({ utility, value }) => (
          <div key={utility} className="flex items-baseline gap-4">
            <code className="timecode w-24 shrink-0 text-xs text-text-muted">{utility}</code>
            <span className="text-md" style={{ fontWeight: value }}>
              Inter {value}
            </span>
          </div>
        ))}
      </section>
      <section className="flex flex-col gap-2">
        <h2 className="text-md font-semibold">Families</h2>
        <p className="font-sans text-md">font-sans — Inter Variable: Exporter la séquence</p>
        <p className="font-mono text-md">font-mono — JetBrains Mono: const fps = 25;</p>
        <p className="text-md">
          <code className="timecode">timecode</code> (mono + tabular numerals):{' '}
          <span className="timecode text-text">00:01:23:12 · 01:11:00:00</span>
        </p>
      </section>
    </div>
  );
}

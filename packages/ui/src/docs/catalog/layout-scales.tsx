import layoutCss from '../../tokens/layout.css?raw';
import motionCss from '../../tokens/motion.css?raw';
import { parseTokenDeclarations } from '../../tokens/token-css';

const layout = parseTokenDeclarations(layoutCss);
const motion = parseTokenDeclarations(motionCss);

const SPACING_STEPS = [1, 2, 3, 4, 6, 8, 12, 16];

function TokenTable({ rows }: { rows: readonly { name: string; value: string }[] }) {
  return (
    <table className="w-full text-left text-xs">
      <tbody>
        {rows.map(({ name, value }) => (
          <tr key={name} className="border-b border-border">
            <th scope="row" className="py-1.5 pr-4 font-normal">
              <code className="timecode text-text">--{name}</code>
            </th>
            <td className="timecode py-1.5 text-text-muted">{value}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

/** The 4 px spacing grid and the control heights, generated from `layout.css`. */
export function SpacingScale() {
  const controls = layout.filter(({ name }) => name.startsWith('spacing-control-'));
  return (
    <div className="flex w-[40rem] max-w-full flex-col gap-6">
      <section className="flex flex-col gap-2">
        <h2 className="text-md font-semibold">Spacing (4 px grid)</h2>
        {SPACING_STEPS.map((step) => (
          <div key={step} className="flex items-center gap-4">
            <code className="timecode w-16 text-xs text-text-muted">p-{step}</code>
            <span className="w-12 text-2xs text-text-subtle">{step * 4} px</span>
            <span
              aria-hidden="true"
              className="h-3 rounded-xs bg-accent"
              style={{ width: `calc(var(--spacing) * ${String(step)})` }}
            />
          </div>
        ))}
      </section>
      <section className="flex flex-col gap-2">
        <h2 className="text-md font-semibold">Control heights</h2>
        <div className="flex items-end gap-3">
          {controls.map(({ name }) => (
            <span
              key={name}
              className="flex items-center rounded-md border border-border-strong bg-control px-3 text-xs"
              style={{ height: `var(--${name})` }}
            >
              h-{name.replace('spacing-', '')}
            </span>
          ))}
        </div>
      </section>
    </div>
  );
}

/** Radii and the overlay shadow, generated from `layout.css`. */
export function RadiusAndElevation() {
  const radii = layout.filter(({ name }) => name.startsWith('radius-'));
  return (
    <div className="flex w-[40rem] max-w-full flex-col gap-6">
      <div className="flex gap-4">
        {radii.map(({ name, value }) => (
          <div key={name} className="flex flex-col items-center gap-1">
            <span
              aria-hidden="true"
              className="size-16 border border-border-strong bg-panel-raised"
              style={{ borderRadius: `var(--${name})` }}
            />
            <code className="timecode text-2xs text-text-muted">rounded-{name.slice(7)}</code>
            <span className="text-2xs text-text-subtle">{value}</span>
          </div>
        ))}
      </div>
      <div className="flex items-center gap-4">
        <span className="rounded-lg bg-overlay p-4 text-xs shadow-overlay">shadow-overlay</span>
        <span className="text-xs text-text-muted">The only shadow: menus, popovers, dialogs.</span>
      </div>
    </div>
  );
}

/** Stacking layers, durations and easings (variables of `layout.css` and `motion.css`). */
export function LayersAndMotion() {
  const rows = [
    ...layout.filter(({ name }) => name.startsWith('z-')),
    ...motion.filter(({ name }) => /^(duration|ease)-/.test(name)),
  ];
  return (
    <div className="w-[32rem] max-w-full">
      <TokenTable rows={rows} />
    </div>
  );
}

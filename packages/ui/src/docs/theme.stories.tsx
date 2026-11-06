import type { Meta, StoryObj } from '@storybook/react-vite';
import { Download, Scissors } from 'lucide-react';

import { Button } from '../primitives/button';
import { Checkbox } from '../primitives/checkbox';
import { Field } from '../primitives/field';
import { Icon } from '../primitives/icon';
import { Progress } from '../primitives/progress';
import { Slider } from '../primitives/slider';
import { Switch } from '../primitives/switch';
import { ToggleGroup, ToggleGroupItem } from '../primitives/toggle-group';

/** Accent shades defined in `tokens/accents.css`, shown side by side; `yellow` is the default. */
const ACCENTS = [
  { id: 'lemon', name: 'Citron', hue: '108°' },
  { id: 'yellow', name: 'Jaune vif (défaut)', hue: '96°' },
  { id: 'gold', name: 'Or', hue: '84°' },
] as const;

const ROWS = ['Interview_A.mp4', 'Plan_large_02.mov', 'Musique_intro.wav'];

function AccentPreview({ id, name, hue }: (typeof ACCENTS)[number]) {
  return (
    <section
      data-accent={id}
      aria-label={name}
      className="flex w-72 flex-col gap-4 rounded-lg border border-border bg-panel p-4"
    >
      <header className="flex items-center justify-between">
        <h2 className="text-md font-semibold">{name}</h2>
        <code className="timecode text-2xs text-text-subtle">
          {id} · {hue}
        </code>
      </header>
      <div className="flex flex-wrap gap-2">
        <Button variant="primary">
          <Icon icon={Download} size="sm" />
          Exporter
        </Button>
        <Button variant="secondary">Annuler</Button>
      </div>
      <ul className="flex flex-col rounded-md border border-border bg-input py-1 text-sm">
        {ROWS.map((row, index) => (
          <li
            key={row}
            aria-current={index === 1 ? 'true' : undefined}
            className="flex h-control-sm items-center gap-2 px-2 aria-current:bg-accent-muted"
          >
            <Icon icon={Scissors} size="sm" className="text-text-muted" />
            {row}
          </li>
        ))}
      </ul>
      <ToggleGroup type="single" defaultValue="edit" aria-label="Mode">
        <ToggleGroupItem value="edit">Montage</ToggleGroupItem>
        <ToggleGroupItem value="audio">Audio</ToggleGroupItem>
        <ToggleGroupItem value="export">Export</ToggleGroupItem>
      </ToggleGroup>
      <div className="flex items-center gap-4">
        <Field label="Magnétisme" orientation="horizontal" id={`${id}-snap`}>
          <Switch defaultChecked />
        </Field>
        <Field label="Proxy" orientation="horizontal" id={`${id}-proxy`}>
          <Checkbox defaultChecked />
        </Field>
      </div>
      <Slider thumbLabels={['Volume']} defaultValue={[70]} />
      <div className="flex items-center gap-3">
        <Progress value={66} aria-label="Export" className="flex-1" />
        <span className="timecode text-xs text-text-muted">66 %</span>
      </div>
      <div className="flex items-center gap-2 text-xs text-text-muted">
        <span className="rounded-sm px-2 py-1 text-text outline-2 outline-offset-1 outline-focus-ring">
          Focus clavier
        </span>
        <span className="rounded-sm bg-accent-muted px-2 py-1 text-text">Sélection</span>
      </div>
    </section>
  );
}

const meta = {
  title: 'Foundations/Theme',
  parameters: { layout: 'padded' },
} satisfies Meta;

export default meta;
type Story = StoryObj<typeof meta>;

/**
 * The three yellow shades on the same dark surfaces: primary action, selection, toggles, slider,
 * progress and the keyboard focus ring. The default accent of `colors.css` is `yellow`.
 */
export const AccentVariants: Story = {
  render: () => (
    <div className="flex flex-wrap gap-4">
      {ACCENTS.map((accent) => (
        <AccentPreview key={accent.id} {...accent} />
      ))}
    </div>
  ),
};

import type { Meta, StoryObj } from '@storybook/react-vite';

import { Label } from '../label';
import { RadioGroup, RadioGroupItem } from './radio-group';

const FORMATS = [
  { value: '16:9', label: '16:9 — Paysage' },
  { value: '9:16', label: '9:16 — Vertical' },
  { value: '1:1', label: '1:1 — Carré' },
  { value: '4:5', label: '4:5 — Portrait' },
];

// Annotated rather than `satisfies`: the inferred type would name Radix-internal prop types,
// which declaration emit cannot reference.
const meta: Meta<typeof RadioGroup> = {
  title: 'Primitives/RadioGroup',
  component: RadioGroup,
  args: { 'aria-label': 'Format de la séquence', defaultValue: '16:9' },
  render: (args) => (
    <RadioGroup {...args}>
      {FORMATS.map((format) => (
        <div key={format.value} className="flex items-center gap-2">
          <RadioGroupItem value={format.value} id={`story-format-${format.value}`} />
          <Label htmlFor={`story-format-${format.value}`}>{format.label}</Label>
        </div>
      ))}
    </RadioGroup>
  ),
};

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Horizontal: Story = {
  args: { orientation: 'horizontal', className: 'flex gap-4' },
};

/** A disabled group dims its labels through the `peer` styles of `Label`. */
export const Disabled: Story = { args: { disabled: true } };

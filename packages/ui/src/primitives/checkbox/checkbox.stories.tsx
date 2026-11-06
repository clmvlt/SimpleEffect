import type { Meta, StoryObj } from '@storybook/react-vite';

import { Field } from '../field';
import { Checkbox } from './checkbox';

const meta = {
  title: 'Primitives/Checkbox',
  component: Checkbox,
  args: { 'aria-label': 'Verrouiller la piste' },
} satisfies Meta<typeof Checkbox>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Checked: Story = { args: { defaultChecked: true } };

/** Partial selection, announced as « mixed ». */
export const Indeterminate: Story = { args: { checked: 'indeterminate' } };

export const Disabled: Story = { args: { disabled: true, defaultChecked: true } };

export const Invalid: Story = { args: { 'aria-invalid': true } };

/** With a visible label, through a horizontal `Field`. */
export const WithField: Story = {
  render: (args) => (
    <div className="flex flex-col gap-2">
      <Field label="Aimanter à la tête de lecture" orientation="horizontal">
        <Checkbox {...args} aria-label={undefined} defaultChecked />
      </Field>
      <Field
        label="Afficher les formes d’onde"
        orientation="horizontal"
        description="Peut ralentir les longues timelines."
      >
        <Checkbox {...args} aria-label={undefined} />
      </Field>
    </div>
  ),
};

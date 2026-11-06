import type { Meta, StoryObj } from '@storybook/react-vite';

import { Field } from '../field';
import { Switch } from './switch';

const meta = {
  title: 'Primitives/Switch',
  component: Switch,
  args: { 'aria-label': 'Proxys de lecture' },
} satisfies Meta<typeof Switch>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const Checked: Story = { args: { defaultChecked: true } };

export const Disabled: Story = { args: { disabled: true } };

export const DisabledChecked: Story = { args: { disabled: true, defaultChecked: true } };

/** With a visible label and a description, through a horizontal `Field`. */
export const WithField: Story = {
  render: (args) => (
    <Field
      label="Proxys de lecture"
      orientation="horizontal"
      description="Lit les copies basse résolution pour une lecture fluide."
    >
      <Switch {...args} aria-label={undefined} defaultChecked />
    </Field>
  ),
};

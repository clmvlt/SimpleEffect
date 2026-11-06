import type { Meta, StoryObj } from '@storybook/react-vite';

import { Checkbox } from '../checkbox';
import { Input } from '../input';
import { Textarea } from '../textarea';
import { Field } from './field';

const meta = {
  title: 'Primitives/Field',
  component: Field,
  args: { label: 'Nom de la séquence', children: <Input placeholder="Séquence 1" /> },
  argTypes: { children: { control: false } },
  decorators: [
    (Story) => (
      <div className="w-72">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Field>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

export const WithDescription: Story = {
  args: { description: 'Visible dans la liste des séquences et dans les exports.' },
};

/** The error marks the control `aria-invalid` and is read with it. */
export const WithError: Story = {
  args: {
    label: 'Durée',
    description: 'Format hh:mm:ss:ff',
    error: 'Timecode invalide : 99 images dépasse la cadence.',
    children: <Input defaultValue="00:00:10:99" className="timecode" />,
  },
};

export const WithTextarea: Story = {
  args: { label: 'Consigne', children: <Textarea placeholder="Décris le montage souhaité…" /> },
};

/** Control first, label after: for checkboxes and switches. */
export const Horizontal: Story = {
  args: {
    label: 'Aimanter à la tête de lecture',
    orientation: 'horizontal',
    children: <Checkbox defaultChecked />,
  },
};

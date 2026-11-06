import type { Meta, StoryObj } from '@storybook/react-vite';

import { Input } from './input';

const meta = {
  title: 'Primitives/Input',
  component: Input,
  args: { 'aria-label': 'Nom de la séquence', placeholder: 'Séquence 1', size: 'sm' },
  argTypes: { size: { control: 'inline-radio', options: ['xs', 'sm', 'md'] } },
  decorators: [
    (Story) => (
      <div className="w-64">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Input>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** The three control heights: 24, 28 and 32 px. */
export const Sizes: Story = {
  render: (args) => (
    <div className="flex flex-col gap-2">
      <Input {...args} size="xs" aria-label="Taille xs" />
      <Input {...args} size="sm" aria-label="Taille sm" />
      <Input {...args} size="md" aria-label="Taille md" />
    </div>
  ),
};

export const WithValue: Story = { args: { defaultValue: 'Interview — plan large' } };

export const Invalid: Story = { args: { 'aria-invalid': true, defaultValue: '25:99:00' } };

export const Disabled: Story = { args: { disabled: true, defaultValue: 'Verrouillé' } };

/** Numeric readouts use the `timecode` utility (monospace, tabular digits). */
export const Timecode: Story = {
  args: { 'aria-label': 'Position', defaultValue: '00:01:24:12', className: 'timecode' },
};

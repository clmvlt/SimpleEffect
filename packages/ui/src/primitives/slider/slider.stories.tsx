import type { Meta, StoryObj } from '@storybook/react-vite';

import { Slider } from './slider';

const meta = {
  title: 'Primitives/Slider',
  component: Slider,
  args: { thumbLabels: ['Volume'], defaultValue: [70], min: 0, max: 100, step: 1 },
  decorators: [
    (Story) => (
      <div className="flex w-64 justify-center">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Slider>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Two thumbs select a range; each is named separately. */
export const Range: Story = { args: { thumbLabels: ['Début', 'Fin'], defaultValue: [20, 65] } };

export const Vertical: Story = {
  args: { orientation: 'vertical', className: 'h-32' },
};

export const Disabled: Story = { args: { disabled: true } };

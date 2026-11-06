import type { Meta, StoryObj } from '@storybook/react-vite';

import { Separator } from './separator';

const meta = {
  title: 'Primitives/Separator',
  component: Separator,
  args: { orientation: 'horizontal', decorative: true },
} satisfies Meta<typeof Separator>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Between stacked groups of a panel. */
export const Horizontal: Story = {
  render: (args) => (
    <div className="flex w-64 flex-col gap-2 text-sm">
      <span className="text-text">Transformations</span>
      <Separator {...args} />
      <span className="text-text-muted">Opacité</span>
    </div>
  ),
};

/** Between groups of toolbar buttons. */
export const Vertical: Story = {
  args: { orientation: 'vertical' },
  render: (args) => (
    <div className="flex h-6 items-center gap-3 text-sm text-text-muted">
      <span>Couper</span>
      <Separator {...args} />
      <span>Scinder</span>
      <Separator {...args} />
      <span>Supprimer</span>
    </div>
  ),
};

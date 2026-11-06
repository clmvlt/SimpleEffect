import type { Meta, StoryObj } from '@storybook/react-vite';
import { Scissors } from 'lucide-react';

import { Button } from '../button';
import { Icon } from '../icon';
import { Tooltip, TooltipProvider } from './tooltip';

const meta = {
  title: 'Primitives/Tooltip',
  component: Tooltip,
  decorators: [
    (Story) => (
      <TooltipProvider>
        <Story />
      </TooltipProvider>
    ),
  ],
  args: {
    content: 'Couper le clip',
    children: (
      <Button variant="ghost" aria-label="Couper">
        <Icon icon={Scissors} size="sm" />
      </Button>
    ),
  },
  argTypes: {
    side: { control: 'inline-radio', options: ['top', 'right', 'bottom', 'left'] },
    children: { control: false },
  },
} satisfies Meta<typeof Tooltip>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Hover or focus the button (Tab) to show the tooltip. */
export const Default: Story = { args: { shortcut: ['Ctrl', 'K'] } };

/** Shown open, to review the surface and the shortcut key caps. */
export const Open: Story = { args: { shortcut: ['Ctrl', 'K'], defaultOpen: true } };

/** A plain label, without shortcut. */
export const WithoutShortcut: Story = {
  args: { content: 'Ajouter un marqueur', defaultOpen: true },
};

/** The four sides. */
export const Sides: Story = {
  render: (args) => (
    <div className="grid grid-cols-2 gap-16 p-12">
      {(['top', 'right', 'bottom', 'left'] as const).map((side) => (
        <Tooltip key={side} {...args} shortcut={['Ctrl', 'K']} side={side} defaultOpen>
          <Button>{side}</Button>
        </Tooltip>
      ))}
    </div>
  ),
};

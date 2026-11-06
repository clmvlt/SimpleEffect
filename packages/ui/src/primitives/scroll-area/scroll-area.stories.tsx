import type { Meta, StoryObj } from '@storybook/react-vite';

import { ScrollArea } from './scroll-area';

const clips = Array.from(
  { length: 40 },
  (_, index) => `Plan ${String(index + 1).padStart(2, '0')}`,
);

const meta = {
  title: 'Primitives/ScrollArea',
  component: ScrollArea,
  args: { scrollbars: 'vertical', keyboardScrollable: true },
  argTypes: {
    scrollbars: { control: 'inline-radio', options: ['vertical', 'horizontal', 'both'] },
  },
} satisfies Meta<typeof ScrollArea>;

export default meta;
type Story = StoryObj<typeof meta>;

/** A list taller than its box. Hover it to reveal the scrollbar. */
export const Vertical: Story = {
  render: (args) => (
    <ScrollArea {...args} className="h-48 w-56 rounded-md border border-border bg-panel">
      <ul className="flex flex-col p-2 text-sm text-text-muted">
        {clips.map((clip) => (
          <li key={clip} className="rounded-sm px-2 py-1 hover:bg-hover hover:text-text">
            {clip}
          </li>
        ))}
      </ul>
    </ScrollArea>
  ),
};

/** A row wider than its box, like a strip of thumbnails. */
export const Horizontal: Story = {
  args: { scrollbars: 'horizontal' },
  render: (args) => (
    <ScrollArea {...args} className="w-80 rounded-md border border-border bg-panel">
      <div className="flex gap-2 p-2">
        {clips.slice(0, 16).map((clip) => (
          <div
            key={clip}
            className="flex aspect-video w-24 shrink-0 items-center justify-center rounded-sm bg-item-video-muted text-2xs text-item-video-strong"
          >
            {clip}
          </div>
        ))}
      </div>
    </ScrollArea>
  ),
};

/** Content larger than the box in both directions. */
export const Both: Story = {
  args: { scrollbars: 'both' },
  render: (args) => (
    <ScrollArea {...args} className="h-48 w-72 rounded-md border border-border bg-panel">
      <div className="grid w-[40rem] grid-cols-8 gap-2 p-2">
        {clips.map((clip) => (
          <div key={clip} className="rounded-sm bg-control px-2 py-3 text-2xs text-text-muted">
            {clip}
          </div>
        ))}
      </div>
    </ScrollArea>
  ),
};

import type { Meta, StoryObj } from '@storybook/react-vite';

import { Skeleton } from './skeleton';

const meta = {
  title: 'Primitives/Skeleton',
  component: Skeleton,
  args: { className: 'h-4 w-48' },
} satisfies Meta<typeof Skeleton>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Line: Story = {};

/** A media bin row while its thumbnail and metadata load. */
export const MediaRow: Story = {
  render: () => (
    <div className="flex w-72 items-center gap-3" aria-busy="true">
      <Skeleton className="aspect-video w-20 rounded-sm" />
      <div className="flex flex-1 flex-col gap-2">
        <Skeleton className="h-3 w-3/4" />
        <Skeleton className="h-3 w-1/3" />
      </div>
    </div>
  ),
};

import type { Meta, StoryObj } from '@storybook/react-vite';

import { Progress } from './progress';

const meta = {
  title: 'Primitives/Progress',
  component: Progress,
  args: { value: 60, size: 'sm', tone: 'accent', 'aria-label': 'Export' },
  argTypes: {
    value: { control: { type: 'range', min: 0, max: 100 } },
    size: { control: 'inline-radio', options: ['xs', 'sm', 'md'] },
    tone: { control: 'inline-radio', options: ['accent', 'success', 'warning', 'danger', 'ai'] },
  },
  decorators: [
    (Story) => (
      <div className="w-72">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof Progress>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Determinate: Story = {};

/** Unknown amount of work: a segment sweeps across the track and no value is announced. */
export const Indeterminate: Story = { args: { value: null, 'aria-label': 'Préparation' } };

/** Sizes and tones, with a visible label linked through `aria-labelledby`. */
export const Variants: Story = {
  render: () => (
    <div className="flex flex-col gap-4">
      {(['accent', 'success', 'warning', 'danger', 'ai'] as const).map((tone, index) => (
        <div key={tone} className="flex flex-col gap-1.5">
          <span id={`progress-${tone}`} className="text-xs text-text-muted">
            {tone}
          </span>
          <Progress
            value={20 + index * 18}
            tone={tone}
            size={index % 3 === 0 ? 'md' : index % 3 === 1 ? 'sm' : 'xs'}
            aria-labelledby={`progress-${tone}`}
          />
        </div>
      ))}
    </div>
  ),
};

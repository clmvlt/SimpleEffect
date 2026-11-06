import type { Meta, StoryObj } from '@storybook/react-vite';

import { StatusDot } from './status-dot';

const TONES = ['neutral', 'success', 'warning', 'danger', 'info', 'accent', 'ai'] as const;

const meta = {
  title: 'Editor/StatusDot',
  component: StatusDot,
  args: { tone: 'success', label: 'Connecté', size: 'sm', pulse: false },
  argTypes: {
    tone: { control: 'select', options: TONES },
    size: { control: 'inline-radio', options: ['xs', 'sm'] },
  },
} satisfies Meta<typeof StatusDot>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Every tone next to its meaning (decorative dots: the text says it). */
export const Tones: Story = {
  render: () => (
    <ul className="flex flex-col gap-2 text-xs text-text-muted">
      {TONES.map((tone) => (
        <li key={tone} className="flex items-center gap-2">
          <StatusDot tone={tone} />
          {tone}
        </li>
      ))}
    </ul>
  ),
};

/** An ongoing state breathes. */
export const Pulsing: Story = { args: { tone: 'warning', label: 'Reconnexion…', pulse: true } };

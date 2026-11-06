import type { Meta, StoryObj } from '@storybook/react-vite';
import { Magnet, Pause, Play, Scissors, Sparkles, Trash2 } from 'lucide-react';

import { IconButton } from './icon-button';

const meta = {
  title: 'Primitives/IconButton',
  component: IconButton,
  args: { icon: Scissors, label: 'Couper le clip', variant: 'ghost', size: 'sm' },
  argTypes: {
    icon: { control: false },
    variant: {
      control: 'inline-radio',
      options: ['primary', 'secondary', 'outline', 'ghost', 'danger'],
    },
    size: { control: 'inline-radio', options: ['xs', 'sm', 'md'] },
  },
} satisfies Meta<typeof IconButton>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Every variant, in every size. */
export const Variants: Story = {
  render: (args) => (
    <div className="flex flex-col gap-3">
      {(['md', 'sm', 'xs'] as const).map((size) => (
        <div key={size} className="flex items-center gap-2">
          <IconButton {...args} size={size} variant="primary" icon={Play} label="Lecture" />
          <IconButton {...args} size={size} variant="secondary" icon={Pause} label="Pause" />
          <IconButton {...args} size={size} variant="outline" icon={Magnet} label="Magnétisme" />
          <IconButton {...args} size={size} variant="ghost" icon={Scissors} label="Couper" />
          <IconButton {...args} size={size} variant="danger" icon={Trash2} label="Supprimer" />
        </div>
      ))}
    </div>
  ),
};

/** A dense toolbar, the main use of icon buttons. */
export const Toolbar: Story = {
  render: (args) => (
    <div className="flex items-center gap-0.5 rounded-md border border-border bg-panel p-0.5">
      <IconButton {...args} icon={Play} label="Lecture" />
      <IconButton {...args} icon={Scissors} label="Couper" />
      <IconButton {...args} icon={Magnet} label="Magnétisme" />
      <IconButton {...args} icon={Sparkles} label="Demander à l’IA" />
    </div>
  ),
};

export const Loading: Story = { args: { loading: true, variant: 'secondary' } };

export const Disabled: Story = { args: { disabled: true } };

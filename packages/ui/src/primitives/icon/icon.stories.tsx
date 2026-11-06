import type { Meta, StoryObj } from '@storybook/react-vite';
import { Film, Music, Scissors, Sparkles, Type } from 'lucide-react';

import { ICON_SIZES, Icon, type IconSize } from './icon';

const meta = {
  title: 'Primitives/Icon',
  component: Icon,
  args: { icon: Scissors, size: 'md' },
  argTypes: { icon: { control: false } },
} satisfies Meta<typeof Icon>;

export default meta;
type Story = StoryObj<typeof meta>;

/** A decorative icon (no `label`): hidden from screen readers. */
export const Default: Story = {};

/** The four normalised sizes: 12, 14, 16 and 20 px. */
export const Sizes: Story = {
  render: (args) => (
    <div className="flex items-end gap-4 text-text">
      {(Object.keys(ICON_SIZES) as IconSize[]).map((size) => (
        <div key={size} className="flex flex-col items-center gap-1">
          <Icon {...args} size={size} />
          <span className="text-2xs text-text-subtle">{size}</span>
        </div>
      ))}
    </div>
  ),
};

/** Icons inherit the text colour, so item-kind tokens colour them. */
export const Colours: Story = {
  render: () => (
    <div className="flex gap-3">
      <Icon icon={Film} className="text-item-video-strong" label="Vidéo" />
      <Icon icon={Music} className="text-item-audio-strong" label="Audio" />
      <Icon icon={Type} className="text-item-graphic-strong" label="Texte" />
      <Icon icon={Sparkles} className="text-ai-strong" label="IA" />
    </div>
  ),
};

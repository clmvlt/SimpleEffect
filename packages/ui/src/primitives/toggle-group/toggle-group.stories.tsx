import type { Meta, StoryObj } from '@storybook/react-vite';
import {
  AlignCenter,
  AlignLeft,
  AlignRight,
  Bold,
  Hand,
  Italic,
  MousePointer2,
  Scissors,
  Underline,
} from 'lucide-react';

import { Icon } from '../icon';
import { ToggleGroup, ToggleGroupItem } from './toggle-group';

const meta = {
  title: 'Primitives/ToggleGroup',
  component: ToggleGroup,
  args: { type: 'single', variant: 'ghost', size: 'sm' },
  argTypes: {
    variant: { control: 'inline-radio', options: ['ghost', 'segmented'] },
    size: { control: 'inline-radio', options: ['xs', 'sm', 'md'] },
  },
} satisfies Meta<typeof ToggleGroup>;

export default meta;
type Story = StoryObj<typeof meta>;

/** Timeline tools: one active at a time, icon-only items labelled with `aria-label`. */
export const Tools: Story = {
  render: ({ variant, size }) => (
    <ToggleGroup
      variant={variant}
      size={size}
      type="single"
      defaultValue="select"
      aria-label="Outils"
    >
      <ToggleGroupItem value="select" aria-label="Sélection">
        <Icon icon={MousePointer2} size="sm" />
      </ToggleGroupItem>
      <ToggleGroupItem value="razor" aria-label="Lame">
        <Icon icon={Scissors} size="sm" />
      </ToggleGroupItem>
      <ToggleGroupItem value="hand" aria-label="Main">
        <Icon icon={Hand} size="sm" />
      </ToggleGroupItem>
    </ToggleGroup>
  ),
};

/** An exclusive choice drawn as a segmented control. */
export const Segmented: Story = {
  args: { variant: 'segmented' },
  render: ({ variant, size }) => (
    <ToggleGroup
      variant={variant}
      size={size}
      type="single"
      defaultValue="center"
      aria-label="Alignement du texte"
    >
      <ToggleGroupItem value="left" aria-label="Aligner à gauche">
        <Icon icon={AlignLeft} size="sm" />
      </ToggleGroupItem>
      <ToggleGroupItem value="center" aria-label="Centrer">
        <Icon icon={AlignCenter} size="sm" />
      </ToggleGroupItem>
      <ToggleGroupItem value="right" aria-label="Aligner à droite">
        <Icon icon={AlignRight} size="sm" />
      </ToggleGroupItem>
    </ToggleGroup>
  ),
};

/** Independent toggles (`type="multiple"`). */
export const Multiple: Story = {
  render: ({ variant, size }) => (
    <ToggleGroup
      variant={variant}
      size={size}
      type="multiple"
      defaultValue={['bold']}
      aria-label="Style du texte"
    >
      <ToggleGroupItem value="bold" aria-label="Gras">
        <Icon icon={Bold} size="sm" />
      </ToggleGroupItem>
      <ToggleGroupItem value="italic" aria-label="Italique">
        <Icon icon={Italic} size="sm" />
      </ToggleGroupItem>
      <ToggleGroupItem value="underline" aria-label="Souligné" disabled>
        <Icon icon={Underline} size="sm" />
      </ToggleGroupItem>
    </ToggleGroup>
  ),
};

/** Text items in every size, both variants. */
export const Sizes: Story = {
  render: () => (
    <div className="flex flex-col gap-3">
      {(['ghost', 'segmented'] as const).map((variant) =>
        (['md', 'sm', 'xs'] as const).map((size) => (
          <ToggleGroup
            key={`${variant}-${size}`}
            type="single"
            variant={variant}
            size={size}
            defaultValue="16:9"
            aria-label={`Format ${variant} ${size}`}
          >
            <ToggleGroupItem value="16:9">16:9</ToggleGroupItem>
            <ToggleGroupItem value="9:16">9:16</ToggleGroupItem>
            <ToggleGroupItem value="1:1">1:1</ToggleGroupItem>
          </ToggleGroup>
        )),
      )}
    </div>
  ),
};

import type { Meta, StoryObj } from '@storybook/react-vite';
import { Download, Plus, Trash2 } from 'lucide-react';

import { Icon } from '../icon';
import { Button } from './button';

const meta = {
  title: 'Primitives/Button',
  component: Button,
  args: { children: 'Exporter', variant: 'secondary', size: 'sm' },
  argTypes: {
    variant: {
      control: 'inline-radio',
      options: ['primary', 'secondary', 'outline', 'ghost', 'danger'],
    },
    size: { control: 'inline-radio', options: ['xs', 'sm', 'md'] },
  },
} satisfies Meta<typeof Button>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Primary: Story = { args: { variant: 'primary' } };

export const Secondary: Story = {};

/** Every variant, in every size. */
export const Variants: Story = {
  render: (args) => (
    <div className="flex flex-col gap-3">
      {(['md', 'sm', 'xs'] as const).map((size) => (
        <div key={size} className="flex items-center gap-2">
          <Button {...args} size={size} variant="primary">
            Primary
          </Button>
          <Button {...args} size={size} variant="secondary">
            Secondary
          </Button>
          <Button {...args} size={size} variant="outline">
            Outline
          </Button>
          <Button {...args} size={size} variant="ghost">
            Ghost
          </Button>
          <Button {...args} size={size} variant="danger">
            Danger
          </Button>
        </div>
      ))}
    </div>
  ),
};

/** An icon before the label; the icon is decorative (the label names the button). */
export const WithIcon: Story = {
  render: (args) => (
    <div className="flex gap-2">
      <Button {...args} variant="primary">
        <Icon icon={Download} size="sm" />
        Exporter
      </Button>
      <Button {...args}>
        <Icon icon={Plus} size="sm" />
        Nouvelle séquence
      </Button>
      <Button {...args} variant="danger">
        <Icon icon={Trash2} size="sm" />
        Supprimer
      </Button>
    </div>
  ),
};

export const Loading: Story = { args: { variant: 'primary', loading: true, children: 'Export…' } };

export const Disabled: Story = { args: { disabled: true } };

/** `asChild` renders a link with the button styles. */
export const AsLink: Story = {
  args: { asChild: true, variant: 'outline', children: <a href="#docs">Documentation</a> },
};

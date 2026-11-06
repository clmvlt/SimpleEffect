import type { Meta, StoryObj } from '@storybook/react-vite';
import { Sparkles } from 'lucide-react';

import { Icon } from '../icon';
import { Badge } from './badge';

const meta = {
  title: 'Primitives/Badge',
  component: Badge,
  args: { children: 'Analysé', variant: 'neutral', size: 'sm', shape: 'rounded' },
  argTypes: {
    variant: {
      control: 'select',
      options: [
        'neutral',
        'accent',
        'success',
        'warning',
        'danger',
        'info',
        'video',
        'audio',
        'graphic',
        'caption',
        'ai',
      ],
    },
    size: { control: 'inline-radio', options: ['xs', 'sm'] },
    shape: { control: 'inline-radio', options: ['rounded', 'pill'] },
  },
} satisfies Meta<typeof Badge>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** Status badges. */
export const Semantic: Story = {
  render: (args) => (
    <div className="flex flex-wrap gap-2">
      <Badge {...args} variant="neutral">
        En attente
      </Badge>
      <Badge {...args} variant="accent">
        Sélection
      </Badge>
      <Badge {...args} variant="success">
        Analysé
      </Badge>
      <Badge {...args} variant="warning">
        Proxy manquant
      </Badge>
      <Badge {...args} variant="danger">
        Erreur
      </Badge>
      <Badge {...args} variant="info">
        Nouveau
      </Badge>
    </div>
  ),
};

/** Timeline item kinds, on a panel surface. */
export const ItemKinds: Story = {
  render: (args) => (
    <div className="flex flex-wrap gap-2 rounded-md bg-panel p-3">
      <Badge {...args} variant="video">
        Vidéo
      </Badge>
      <Badge {...args} variant="audio">
        Audio
      </Badge>
      <Badge {...args} variant="graphic">
        Texte
      </Badge>
      <Badge {...args} variant="caption">
        Sous-titres
      </Badge>
      <Badge {...args} variant="ai">
        <Icon icon={Sparkles} size="xs" />
        IA
      </Badge>
    </div>
  ),
};

/** Sizes and shapes. */
export const SizesAndShapes: Story = {
  render: (args) => (
    <div className="flex items-center gap-2">
      <Badge {...args} size="xs">
        xs
      </Badge>
      <Badge {...args} size="sm">
        sm
      </Badge>
      <Badge {...args} size="xs" shape="pill">
        12
      </Badge>
      <Badge {...args} size="sm" shape="pill" variant="accent">
        128
      </Badge>
    </div>
  ),
};

import type { Meta, StoryObj } from '@storybook/react-vite';
import { Clapperboard, Film, Plus } from 'lucide-react';

import { Button } from '../../primitives/button';
import { Icon } from '../../primitives/icon';
import { EmptyState } from './empty-state';

const meta = {
  title: 'Editor/EmptyState',
  component: EmptyState,
  decorators: [
    (Story) => (
      <div className="h-64 w-80 rounded-md border border-border bg-panel">
        <Story />
      </div>
    ),
  ],
  args: {
    icon: Film,
    title: 'Aucun média',
    description: 'Importez des vidéos, des sons ou des images pour commencer.',
  },
} satisfies Meta<typeof EmptyState>;

export default meta;
type Story = StoryObj<typeof meta>;

/** In a panel. */
export const Default: Story = {};

/** With the action that fills the area. */
export const WithAction: Story = {
  args: {
    actions: (
      <Button size="xs">
        <Icon icon={Plus} size="sm" />
        Importer
      </Button>
    ),
  },
};

/** Full-screen size, as on the home page. */
export const Large: Story = {
  args: {
    size: 'md',
    icon: Clapperboard,
    title: 'Aucun projet',
    description: 'Créez un projet pour commencer à monter.',
    actions: <Button variant="primary">Nouveau projet</Button>,
  },
  decorators: [
    (Story) => (
      <div className="h-96 w-[32rem]">
        <Story />
      </div>
    ),
  ],
};

export const TitleOnly: Story = { render: () => <EmptyState title="Aucun média" /> };

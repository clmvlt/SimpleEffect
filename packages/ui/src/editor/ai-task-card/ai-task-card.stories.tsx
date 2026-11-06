import type { Meta, StoryObj } from '@storybook/react-vite';

import { Button } from '../../primitives/button';
import { AiTaskCard } from './ai-task-card';

const FRENCH_LABELS = {
  status: {
    queued: 'En attente',
    running: 'En cours',
    completed: 'Terminée',
    failed: 'Échec',
    cancelled: 'Annulée',
  },
  log: 'Journal de la tâche',
  progress: 'Progression de la tâche',
};

const LOG = [
  '→ lecture de la timeline (3 pistes, 42 clips)',
  '→ détection des silences > 0,8 s',
  '→ 12 coupes proposées',
].join('\n');

const meta = {
  title: 'Editor/AiTaskCard',
  component: AiTaskCard,
  decorators: [
    (Story) => (
      <div className="w-80">
        <Story />
      </div>
    ),
  ],
  args: {
    status: 'running',
    title: 'Retire les silences de la séquence principale',
    meta: '0:42',
    labels: FRENCH_LABELS,
  },
  argTypes: {
    status: {
      control: 'select',
      options: ['queued', 'running', 'completed', 'failed', 'cancelled'],
    },
  },
} satisfies Meta<typeof AiTaskCard>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Queued: Story = {
  args: { status: 'queued', meta: undefined, actions: <Button size="xs">Annuler</Button> },
};

/** Streaming log and indeterminate progress. */
export const Running: Story = {
  args: { log: LOG, actions: <Button size="xs">Annuler</Button> },
};

export const Completed: Story = {
  args: {
    status: 'completed',
    log: `${LOG}\n✓ 12 coupes appliquées`,
    cost: '0,04 $ · 18 k jetons',
    actions: (
      <Button size="xs" variant="primary">
        Voir les changements
      </Button>
    ),
  },
};

export const Failed: Story = {
  args: {
    status: 'failed',
    error: 'Claude Code a quitté avec le code 1 : authentification requise.',
    actions: <Button size="xs">Relancer</Button>,
  },
};

export const Cancelled: Story = { args: { status: 'cancelled', cost: '0,01 $' } };

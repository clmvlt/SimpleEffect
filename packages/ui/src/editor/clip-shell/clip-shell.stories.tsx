import type { Meta, StoryObj } from '@storybook/react-vite';

import { ClipBadge, ClipShell } from './clip-shell';

const meta = {
  title: 'Editor/ClipShell',
  component: ClipShell,
  args: { tone: 'video', label: 'Plage 4K.mp4', selected: false, dimmed: false },
  argTypes: {
    tone: { control: 'select', options: ['video', 'audio', 'graphic', 'caption'] },
  },
  decorators: [
    (Story) => (
      <div className="h-14 w-64">
        <Story />
      </div>
    ),
  ],
} satisfies Meta<typeof ClipShell>;

export default meta;
type Story = StoryObj<typeof meta>;

export const Default: Story = {};

/** One clip per kind of item, then the selected and dimmed states. */
export const Kinds: Story = {
  render: () => (
    <div className="flex flex-col gap-1">
      <ClipShell tone="video" label="Plage 4K.mp4" className="h-12" />
      <ClipShell tone="audio" label="Interview.wav" className="h-12" />
      <ClipShell tone="graphic" label="Titre principal" className="h-8" />
      <ClipShell tone="caption" label="Bonjour à tous" className="h-8" />
      <ClipShell tone="video" label="Sélectionné" selected className="h-8" />
      <ClipShell tone="audio" label="Piste muette" dimmed className="h-8" />
    </div>
  ),
};

/** An item made by an AI task carries the AI badge. */
export const MadeByAi: Story = {
  args: {
    tone: 'graphic',
    label: 'Sous-titre animé',
    badges: <ClipBadge label="IA" description="Créé par l’IA" />,
  },
};

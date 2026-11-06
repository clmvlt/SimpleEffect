import type { Meta, StoryObj } from '@storybook/react-vite';

import { Kbd } from './kbd';

const meta = {
  title: 'Primitives/Kbd',
  component: Kbd,
  args: { children: 'K' },
} satisfies Meta<typeof Kbd>;

export default meta;
type Story = StoryObj<typeof meta>;

/** A single key. */
export const Default: Story = {};

/** A key combination: one key cap per key. */
export const Shortcut: Story = { args: { keys: ['Ctrl', 'Maj', 'E'] } };

/** Shortcuts as they appear next to a label. */
export const InContext: Story = {
  render: () => (
    <ul className="flex w-56 flex-col gap-2 text-sm text-text-muted">
      <li className="flex items-center justify-between">
        Lecture / pause <Kbd>Espace</Kbd>
      </li>
      <li className="flex items-center justify-between">
        Couper <Kbd keys={['Ctrl', 'K']} />
      </li>
      <li className="flex items-center justify-between">
        Annuler <Kbd keys={['Ctrl', 'Z']} />
      </li>
    </ul>
  ),
};

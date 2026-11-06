import type { Meta, StoryObj } from '@storybook/react-vite';

import { Checkbox } from '../checkbox';
import { Input } from '../input';
import { Label } from './label';

const meta = {
  title: 'Primitives/Label',
  component: Label,
  args: { children: 'Nom de la séquence', htmlFor: 'label-story-input' },
} satisfies Meta<typeof Label>;

export default meta;
type Story = StoryObj<typeof meta>;

/** A label linked to its control through `htmlFor` (`Field` does this wiring for you). */
export const Default: Story = {
  render: (args) => (
    <div className="flex w-64 flex-col gap-1.5">
      <Label {...args} />
      <Input id="label-story-input" />
    </div>
  ),
};

/** Dimmed when the preceding control (`peer`) is disabled. */
export const DisabledPeer: Story = {
  render: (args) => (
    <div className="flex items-center gap-2">
      <Checkbox id="label-story-disabled" disabled className="peer" />
      <Label {...args} htmlFor="label-story-disabled">
        Verrouiller la piste
      </Label>
    </div>
  ),
};

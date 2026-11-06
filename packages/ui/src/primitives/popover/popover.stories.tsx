import type { Meta, StoryObj } from '@storybook/react-vite';
import { Gauge } from 'lucide-react';

import { Button } from '../button';
import { Field } from '../field';
import { Icon } from '../icon';
import { Input } from '../input';
import { Popover, PopoverClose, PopoverContent, PopoverTrigger } from './popover';

// Annotated (not `satisfies`): the inferred type would name Radix internals in declarations.
const meta: Meta<typeof Popover> = {
  title: 'Primitives/Popover',
  component: Popover,
};

export default meta;
type Story = StoryObj<typeof Popover>;

function SpeedForm() {
  return (
    <div className="flex flex-col gap-3">
      <p className="text-sm font-medium">Vitesse du clip</p>
      <Field label="Pourcentage">
        <Input size="xs" defaultValue="100" inputMode="numeric" className="timecode text-right" />
      </Field>
      <div className="flex justify-end">
        <PopoverClose asChild>
          <Button size="xs" variant="primary">
            Appliquer
          </Button>
        </PopoverClose>
      </div>
    </div>
  );
}

/** Click the trigger; Escape or a click outside closes the popover. */
export const Default: Story = {
  render: (args) => (
    <Popover {...args}>
      <PopoverTrigger asChild>
        <Button>
          <Icon icon={Gauge} size="sm" />
          Vitesse
        </Button>
      </PopoverTrigger>
      <PopoverContent aria-label="Vitesse du clip">
        <SpeedForm />
      </PopoverContent>
    </Popover>
  ),
};

/** Shown open, to review the surface. */
export const Open: Story = { ...Default, args: { defaultOpen: true } };

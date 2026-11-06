import type { Meta, StoryObj } from '@storybook/react-vite';
import { Copy, Ellipsis, Pencil, Scissors, Trash2 } from 'lucide-react';
import { useState } from 'react';

import { Button } from '../button';
import { Icon } from '../icon';
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuSub,
  DropdownMenuSubContent,
  DropdownMenuSubTrigger,
  DropdownMenuTrigger,
} from './dropdown-menu';

// Annotated (not `satisfies`): the inferred type would name Radix internals in declarations.
const meta: Meta<typeof DropdownMenu> = {
  title: 'Primitives/DropdownMenu',
  component: DropdownMenu,
};

export default meta;
type Story = StoryObj<typeof DropdownMenu>;

function ClipMenuItems() {
  const [snapping, setSnapping] = useState(true);
  const [waveforms, setWaveforms] = useState(false);
  const [rate, setRate] = useState('25');
  return (
    <>
      <DropdownMenuGroup>
        <DropdownMenuItem>
          <Icon icon={Scissors} size="sm" />
          Couper <DropdownMenuShortcut>Ctrl+K</DropdownMenuShortcut>
        </DropdownMenuItem>
        <DropdownMenuItem>
          <Icon icon={Copy} size="sm" />
          Dupliquer <DropdownMenuShortcut>Ctrl+D</DropdownMenuShortcut>
        </DropdownMenuItem>
        <DropdownMenuItem disabled>
          <Icon icon={Pencil} size="sm" />
          Renommer
        </DropdownMenuItem>
      </DropdownMenuGroup>
      <DropdownMenuSeparator />
      <DropdownMenuCheckboxItem checked={snapping} onCheckedChange={setSnapping}>
        Magnétisme
      </DropdownMenuCheckboxItem>
      <DropdownMenuCheckboxItem checked={waveforms} onCheckedChange={setWaveforms}>
        Formes d’onde
      </DropdownMenuCheckboxItem>
      <DropdownMenuSeparator />
      <DropdownMenuLabel inset>Cadence</DropdownMenuLabel>
      <DropdownMenuRadioGroup value={rate} onValueChange={setRate}>
        <DropdownMenuRadioItem value="25">25 i/s</DropdownMenuRadioItem>
        <DropdownMenuRadioItem value="30">30 i/s</DropdownMenuRadioItem>
        <DropdownMenuRadioItem value="60">60 i/s</DropdownMenuRadioItem>
      </DropdownMenuRadioGroup>
      <DropdownMenuSeparator />
      <DropdownMenuSub>
        <DropdownMenuSubTrigger>Exporter</DropdownMenuSubTrigger>
        <DropdownMenuSubContent>
          <DropdownMenuItem>Vidéo MP4</DropdownMenuItem>
          <DropdownMenuItem>Projet Premiere (XML)</DropdownMenuItem>
          <DropdownMenuItem>Sous-titres SRT</DropdownMenuItem>
        </DropdownMenuSubContent>
      </DropdownMenuSub>
      <DropdownMenuSeparator />
      <DropdownMenuItem variant="danger">
        <Icon icon={Trash2} size="sm" />
        Supprimer <DropdownMenuShortcut>Suppr</DropdownMenuShortcut>
      </DropdownMenuItem>
    </>
  );
}

/** Items with icons and shortcuts, checkbox and radio items, a submenu and a destructive item. */
export const Default: Story = {
  render: (args) => (
    <DropdownMenu {...args}>
      <DropdownMenuTrigger asChild>
        <Button variant="ghost" aria-label="Actions du clip">
          <Icon icon={Ellipsis} size="sm" />
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="start">
        <ClipMenuItems />
      </DropdownMenuContent>
    </DropdownMenu>
  ),
};

/** Shown open, to review the surface and every kind of item. */
export const Open: Story = { ...Default, args: { defaultOpen: true, modal: false } };

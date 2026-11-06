import type { Meta, StoryObj } from '@storybook/react-vite';
import { Copy, Scissors, Trash2 } from 'lucide-react';
import { useState } from 'react';

import { Icon } from '../icon';
import {
  ContextMenu,
  ContextMenuCheckboxItem,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuLabel,
  ContextMenuRadioGroup,
  ContextMenuRadioItem,
  ContextMenuSeparator,
  ContextMenuShortcut,
  ContextMenuSub,
  ContextMenuSubContent,
  ContextMenuSubTrigger,
  ContextMenuTrigger,
} from './context-menu';

// Annotated (not `satisfies`): the inferred type would name Radix internals in declarations.
const meta: Meta<typeof ContextMenu> = {
  title: 'Primitives/ContextMenu',
  component: ContextMenu,
};

export default meta;
type Story = StoryObj<typeof ContextMenu>;

function ClipContextMenu() {
  const [enabled, setEnabled] = useState(true);
  const [speed, setSpeed] = useState('1');
  return (
    <ContextMenuContent>
      <ContextMenuItem>
        <Icon icon={Scissors} size="sm" />
        Scinder à la tête de lecture <ContextMenuShortcut>Ctrl+K</ContextMenuShortcut>
      </ContextMenuItem>
      <ContextMenuItem>
        <Icon icon={Copy} size="sm" />
        Dupliquer <ContextMenuShortcut>Ctrl+D</ContextMenuShortcut>
      </ContextMenuItem>
      <ContextMenuSeparator />
      <ContextMenuCheckboxItem checked={enabled} onCheckedChange={setEnabled}>
        Activé
      </ContextMenuCheckboxItem>
      <ContextMenuSub>
        <ContextMenuSubTrigger inset>Vitesse</ContextMenuSubTrigger>
        <ContextMenuSubContent>
          <ContextMenuLabel inset>Vitesse de lecture</ContextMenuLabel>
          <ContextMenuRadioGroup value={speed} onValueChange={setSpeed}>
            <ContextMenuRadioItem value="0.5">50 %</ContextMenuRadioItem>
            <ContextMenuRadioItem value="1">100 %</ContextMenuRadioItem>
            <ContextMenuRadioItem value="2">200 %</ContextMenuRadioItem>
          </ContextMenuRadioGroup>
        </ContextMenuSubContent>
      </ContextMenuSub>
      <ContextMenuSeparator />
      <ContextMenuItem variant="danger">
        <Icon icon={Trash2} size="sm" />
        Supprimer <ContextMenuShortcut>Suppr</ContextMenuShortcut>
      </ContextMenuItem>
    </ContextMenuContent>
  );
}

/** Right-click the clip (or focus it and press Shift+F10) to open its menu. */
export const Default: Story = {
  render: (args) => (
    <ContextMenu {...args}>
      <ContextMenuTrigger asChild>
        <div
          tabIndex={0}
          className="flex h-12 w-72 items-center rounded-md border border-item-video-strong bg-item-video-muted px-3 text-sm text-text"
        >
          Interview — plan large
        </div>
      </ContextMenuTrigger>
      <ClipContextMenu />
    </ContextMenu>
  ),
};

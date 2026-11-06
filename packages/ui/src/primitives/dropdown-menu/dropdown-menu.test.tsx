import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';

import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuShortcut,
  DropdownMenuTrigger,
} from './dropdown-menu';

interface Handlers {
  onDuplicate?: () => void;
  onRename?: () => void;
  onRateChange?: (value: string) => void;
}

function EditMenu({ onDuplicate, onRename, onRateChange }: Handlers) {
  const [snapping, setSnapping] = useState(true);
  return (
    <DropdownMenu>
      <DropdownMenuTrigger>Édition</DropdownMenuTrigger>
      <DropdownMenuContent>
        <DropdownMenuItem onSelect={onDuplicate}>
          Dupliquer <DropdownMenuShortcut>Ctrl+D</DropdownMenuShortcut>
        </DropdownMenuItem>
        <DropdownMenuItem disabled>Coller</DropdownMenuItem>
        <DropdownMenuItem onSelect={onRename}>Renommer</DropdownMenuItem>
        <DropdownMenuSeparator />
        <DropdownMenuCheckboxItem checked={snapping} onCheckedChange={setSnapping}>
          Magnétisme
        </DropdownMenuCheckboxItem>
        <DropdownMenuLabel>Cadence</DropdownMenuLabel>
        <DropdownMenuRadioGroup value="25" onValueChange={onRateChange}>
          <DropdownMenuRadioItem value="25">25 i/s</DropdownMenuRadioItem>
          <DropdownMenuRadioItem value="30">30 i/s</DropdownMenuRadioItem>
        </DropdownMenuRadioGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

/** The trigger; `hidden` because an open modal menu hides the rest of the page from assistive tech. */
function trigger() {
  return screen.getByRole('button', { name: 'Édition', hidden: true });
}

describe('DropdownMenu', () => {
  it('opens on click and lists its items', async () => {
    const user = userEvent.setup();
    render(<EditMenu />);
    expect(trigger()).toHaveAttribute('aria-expanded', 'false');

    await user.click(trigger());
    expect(screen.getByRole('menu')).toBeInTheDocument();
    expect(trigger()).toHaveAttribute('aria-expanded', 'true');
    expect(screen.getByRole('menuitem', { name: 'Dupliquer Ctrl+D' })).toBeInTheDocument();
  });

  it.each(['{Enter}', ' '])(
    'opens from the keyboard (%s) with focus on the first item',
    async (key) => {
      const user = userEvent.setup();
      render(<EditMenu />);

      await user.tab();
      await user.keyboard(key);
      expect(screen.getByRole('menuitem', { name: 'Dupliquer Ctrl+D' })).toHaveFocus();
    },
  );

  it('moves with the arrows, skips disabled items and selects with Enter', async () => {
    const user = userEvent.setup();
    const onDuplicate = vi.fn();
    const onRename = vi.fn();
    render(<EditMenu onDuplicate={onDuplicate} onRename={onRename} />);

    await user.tab();
    await user.keyboard('{Enter}');
    await user.keyboard('{ArrowDown}');
    expect(screen.getByRole('menuitem', { name: 'Renommer' })).toHaveFocus();
    await user.keyboard('{Enter}');

    expect(onRename).toHaveBeenCalledOnce();
    expect(onDuplicate).not.toHaveBeenCalled();
    expect(screen.queryByRole('menu')).not.toBeInTheDocument();
    expect(trigger()).toHaveFocus();
  });

  it('closes on Escape and gives focus back to the trigger', async () => {
    const user = userEvent.setup();
    render(<EditMenu />);

    await user.click(trigger());
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('menu')).not.toBeInTheDocument();
    expect(trigger()).toHaveFocus();
  });

  it('toggles a checkbox item', async () => {
    const user = userEvent.setup();
    render(<EditMenu />);

    await user.click(trigger());
    const snapping = screen.getByRole('menuitemcheckbox', { name: 'Magnétisme' });
    expect(snapping).toHaveAttribute('aria-checked', 'true');
    await user.click(snapping);

    await user.click(trigger());
    expect(screen.getByRole('menuitemcheckbox', { name: 'Magnétisme' })).toHaveAttribute(
      'aria-checked',
      'false',
    );
  });

  it('reports the chosen radio item', async () => {
    const user = userEvent.setup();
    const onRateChange = vi.fn();
    render(<EditMenu onRateChange={onRateChange} />);

    await user.click(trigger());
    expect(screen.getByRole('menuitemradio', { name: '25 i/s' })).toHaveAttribute(
      'aria-checked',
      'true',
    );
    await user.click(screen.getByRole('menuitemradio', { name: '30 i/s' }));
    expect(onRateChange).toHaveBeenCalledWith('30');
  });
});

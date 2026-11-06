import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import {
  ContextMenu,
  ContextMenuCheckboxItem,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from './context-menu';

function renderClipArea(onSplit = vi.fn(), onDelete = vi.fn()) {
  render(
    <ContextMenu>
      <ContextMenuTrigger>
        <div>Clip « Interview »</div>
      </ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuItem onSelect={onSplit}>Scinder</ContextMenuItem>
        <ContextMenuCheckboxItem checked>Activé</ContextMenuCheckboxItem>
        <ContextMenuSeparator />
        <ContextMenuItem variant="danger" onSelect={onDelete}>
          Supprimer
        </ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>,
  );
  return { onSplit, onDelete };
}

describe('ContextMenu', () => {
  it('stays closed until the area is right-clicked', () => {
    renderClipArea();
    expect(screen.queryByRole('menu')).not.toBeInTheDocument();

    fireEvent.contextMenu(screen.getByText('Clip « Interview »'));
    expect(screen.getByRole('menu')).toBeInTheDocument();
    expect(screen.getByRole('menuitemcheckbox', { name: 'Activé' })).toHaveAttribute(
      'aria-checked',
      'true',
    );
  });

  it('selects an item with the keyboard and closes', async () => {
    const user = userEvent.setup();
    const { onSplit, onDelete } = renderClipArea();

    fireEvent.contextMenu(screen.getByText('Clip « Interview »'));
    await user.keyboard('{ArrowDown}{ArrowDown}{ArrowDown}');
    const deleteItem = screen.getByRole('menuitem', { name: 'Supprimer' });
    expect(deleteItem).toHaveFocus();
    expect(deleteItem).toHaveClass('text-danger');
    await user.keyboard('{Enter}');

    expect(onDelete).toHaveBeenCalledOnce();
    expect(onSplit).not.toHaveBeenCalled();
    expect(screen.queryByRole('menu')).not.toBeInTheDocument();
  });

  it('closes on Escape without selecting', async () => {
    const user = userEvent.setup();
    const { onSplit } = renderClipArea();

    fireEvent.contextMenu(screen.getByText('Clip « Interview »'));
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('menu')).not.toBeInTheDocument();
    expect(onSplit).not.toHaveBeenCalled();
  });
});

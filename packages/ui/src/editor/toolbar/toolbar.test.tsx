import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Magnet, Scissors, Undo2 } from 'lucide-react';
import { describe, expect, it, vi } from 'vitest';

import { Toolbar, ToolbarButton, ToolbarGroup, ToolbarSeparator } from './toolbar';

function EditingToolbar({ onCut }: { onCut?: () => void }) {
  return (
    <Toolbar label="Outils">
      <ToolbarGroup>
        <ToolbarButton icon={Undo2} label="Annuler" />
        <ToolbarButton icon={Scissors} label="Couper" onClick={onCut} />
      </ToolbarGroup>
      <ToolbarSeparator />
      <ToolbarGroup>
        <ToolbarButton icon={Magnet} label="Magnétisme" pressed />
      </ToolbarGroup>
    </Toolbar>
  );
}

describe('Toolbar', () => {
  it('is a single tab stop navigated with the arrow keys', async () => {
    const user = userEvent.setup();
    render(
      <>
        <EditingToolbar />
        <button type="button">Après</button>
      </>,
    );
    expect(screen.getByRole('toolbar', { name: 'Outils' })).toBeInTheDocument();

    await user.tab();
    expect(screen.getByRole('button', { name: 'Annuler' })).toHaveFocus();
    await user.keyboard('{ArrowRight}');
    expect(screen.getByRole('button', { name: 'Couper' })).toHaveFocus();
    await user.keyboard('{End}');
    expect(screen.getByRole('button', { name: 'Magnétisme' })).toHaveFocus();
    await user.tab();
    expect(screen.getByRole('button', { name: 'Après' })).toHaveFocus();
  });

  it('activates tools and exposes toggles as pressed buttons', async () => {
    const user = userEvent.setup();
    const onCut = vi.fn();
    render(<EditingToolbar onCut={onCut} />);
    await user.click(screen.getByRole('button', { name: 'Couper' }));
    expect(onCut).toHaveBeenCalledOnce();
    expect(screen.getByRole('button', { name: 'Magnétisme' })).toHaveAttribute(
      'aria-pressed',
      'true',
    );
    expect(screen.getByRole('button', { name: 'Couper' })).not.toHaveAttribute('aria-pressed');
  });
});

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Scissors } from 'lucide-react';
import { describe, expect, it, vi } from 'vitest';

import { IconButton } from './icon-button';

describe('IconButton', () => {
  it('is named by its label and reacts to clicks', async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    render(<IconButton icon={Scissors} label="Couper le clip" onClick={onClick} />);

    const button = screen.getByRole('button', { name: 'Couper le clip' });
    expect(button).toHaveAttribute('type', 'button');
    expect(button).toHaveClass('w-control-sm', 'h-control-sm');
    await user.click(button);
    expect(onClick).toHaveBeenCalledOnce();
  });

  it('shows a spinner, is busy and cannot be clicked while loading', async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    render(<IconButton icon={Scissors} label="Couper le clip" loading onClick={onClick} />);

    const button = screen.getByRole('button', { name: 'Couper le clip' });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('aria-busy', 'true');
    expect(button.querySelector('.animate-spin')).not.toBeNull();
    await user.click(button);
    expect(onClick).not.toHaveBeenCalled();
  });
});

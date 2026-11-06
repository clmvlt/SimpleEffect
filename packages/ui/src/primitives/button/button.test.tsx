import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { Button } from './button';

describe('Button', () => {
  it('is a non-submitting button that reacts to clicks and the keyboard', async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Exporter</Button>);

    const button = screen.getByRole('button', { name: 'Exporter' });
    expect(button).toHaveAttribute('type', 'button');
    await user.click(button);
    await user.keyboard('{Enter}');
    await user.keyboard(' ');
    expect(onClick).toHaveBeenCalledTimes(3);
  });

  it('is busy and disabled while loading', async () => {
    const user = userEvent.setup();
    const onClick = vi.fn();
    render(
      <Button loading onClick={onClick}>
        Exporter
      </Button>,
    );

    const button = screen.getByRole('button', { name: 'Exporter' });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('aria-busy', 'true');
    await user.click(button);
    expect(onClick).not.toHaveBeenCalled();
  });

  it('applies the variant and size classes', () => {
    render(
      <Button variant="danger" size="md">
        Supprimer
      </Button>,
    );
    const button = screen.getByRole('button', { name: 'Supprimer' });
    expect(button).toHaveClass('bg-danger-solid', 'h-control-md');
  });

  it('renders its child instead of a button with asChild', () => {
    render(
      <Button asChild variant="ghost">
        <a href="#aide">Aide</a>
      </Button>,
    );
    const link = screen.getByRole('link', { name: 'Aide' });
    expect(link).toHaveClass('hover:bg-hover');
    expect(link).not.toHaveAttribute('type');
  });
});

import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { Popover, PopoverClose, PopoverContent, PopoverTrigger } from './popover';

function renderPopover() {
  return render(
    <Popover>
      <PopoverTrigger>Vitesse</PopoverTrigger>
      <PopoverContent aria-label="Vitesse du clip">
        <label>
          Pourcentage
          <input defaultValue="100" />
        </label>
        <PopoverClose>Appliquer</PopoverClose>
      </PopoverContent>
    </Popover>,
  );
}

describe('Popover', () => {
  it('opens on click, moves focus inside and closes on Escape', async () => {
    const user = userEvent.setup();
    renderPopover();
    const trigger = screen.getByRole('button', { name: 'Vitesse' });
    expect(trigger).toHaveAttribute('aria-expanded', 'false');

    await user.click(trigger);
    const dialog = screen.getByRole('dialog', { name: 'Vitesse du clip' });
    expect(trigger).toHaveAttribute('aria-expanded', 'true');
    expect(dialog).toContainElement(document.activeElement as HTMLElement | null);

    await user.keyboard('{Escape}');
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(trigger).toHaveFocus();
  });

  it('closes through PopoverClose', async () => {
    const user = userEvent.setup();
    renderPopover();

    await user.click(screen.getByRole('button', { name: 'Vitesse' }));
    await user.click(screen.getByRole('button', { name: 'Appliquer' }));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });
});

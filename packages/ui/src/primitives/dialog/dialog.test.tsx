import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from './dialog';

function renderDialog() {
  return render(
    <Dialog>
      <DialogTrigger>Nouvelle séquence</DialogTrigger>
      <DialogContent closeLabel="Fermer">
        <DialogHeader>
          <DialogTitle>Nouvelle séquence</DialogTitle>
          <DialogDescription>Choisis le nom et le format.</DialogDescription>
        </DialogHeader>
        <label>
          Nom
          <input />
        </label>
        <DialogFooter>
          <DialogClose>Annuler</DialogClose>
          <button type="button">Créer</button>
        </DialogFooter>
      </DialogContent>
    </Dialog>,
  );
}

function expectFocusInside(container: HTMLElement) {
  expect(container).toContainElement(document.activeElement as HTMLElement | null);
}

describe('Dialog', () => {
  it('opens from its trigger, named by its title, with focus inside', async () => {
    const user = userEvent.setup();
    renderDialog();
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: 'Nouvelle séquence' }));
    const dialog = screen.getByRole('dialog', { name: 'Nouvelle séquence' });
    expect(dialog).toHaveAccessibleDescription('Choisis le nom et le format.');
    expect(screen.getByRole('textbox', { name: 'Nom' })).toHaveFocus();
  });

  it.each(['{Enter}', ' '])('opens from the keyboard (%s)', async (key) => {
    const user = userEvent.setup();
    renderDialog();

    await user.tab();
    await user.keyboard(key);
    expectFocusInside(screen.getByRole('dialog'));
  });

  it('traps Tab inside the dialog', async () => {
    const user = userEvent.setup();
    renderDialog();
    await user.click(screen.getByRole('button', { name: 'Nouvelle séquence' }));
    const dialog = screen.getByRole('dialog');

    // Input, Annuler, Créer, Fermer: cycling further must wrap around inside the dialog.
    for (let step = 0; step < 6; step += 1) {
      await user.tab();
      expectFocusInside(dialog);
    }
    await user.tab({ shift: true });
    expectFocusInside(dialog);
  });

  it('closes on Escape and gives focus back to the trigger', async () => {
    const user = userEvent.setup();
    renderDialog();
    const trigger = screen.getByRole('button', { name: 'Nouvelle séquence' });

    await user.click(trigger);
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
    expect(trigger).toHaveFocus();
  });

  it('closes through the labelled close button and DialogClose', async () => {
    const user = userEvent.setup();
    renderDialog();
    const trigger = screen.getByRole('button', { name: 'Nouvelle séquence' });

    await user.click(trigger);
    await user.click(screen.getByRole('button', { name: 'Fermer' }));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();

    await user.click(trigger);
    await user.click(screen.getByRole('button', { name: 'Annuler' }));
    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('shows no close button without closeLabel', async () => {
    const user = userEvent.setup();
    render(
      <Dialog>
        <DialogTrigger>Ouvrir</DialogTrigger>
        <DialogContent aria-describedby={undefined}>
          <DialogTitle>Infos</DialogTitle>
        </DialogContent>
      </Dialog>,
    );

    await user.click(screen.getByRole('button', { name: 'Ouvrir' }));
    expect(screen.getByRole('dialog')).toBeInTheDocument();
    expect(screen.queryByRole('button', { name: 'Fermer' })).not.toBeInTheDocument();
  });
});

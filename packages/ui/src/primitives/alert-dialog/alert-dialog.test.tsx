import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from './alert-dialog';

function renderAlertDialog(onConfirm = vi.fn(), onCancel = vi.fn()) {
  render(
    <AlertDialog>
      <AlertDialogTrigger>Supprimer le clip</AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Supprimer le clip ?</AlertDialogTitle>
          <AlertDialogDescription>Cette action est définitive.</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={onCancel}>Annuler</AlertDialogCancel>
          <AlertDialogAction variant="danger" onClick={onConfirm}>
            Supprimer
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>,
  );
  return { onConfirm, onCancel };
}

async function open(user: ReturnType<typeof userEvent.setup>) {
  await user.click(screen.getByRole('button', { name: 'Supprimer le clip' }));
  return screen.getByRole('alertdialog', { name: 'Supprimer le clip ?' });
}

describe('AlertDialog', () => {
  it('is an alert dialog with focus on the cancel button', async () => {
    const user = userEvent.setup();
    renderAlertDialog();

    const dialog = await open(user);
    expect(dialog).toHaveAccessibleDescription('Cette action est définitive.');
    expect(screen.getByRole('button', { name: 'Annuler' })).toHaveFocus();
  });

  it('cancels without confirming', async () => {
    const user = userEvent.setup();
    const { onConfirm, onCancel } = renderAlertDialog();

    await open(user);
    await user.click(screen.getByRole('button', { name: 'Annuler' }));
    expect(onCancel).toHaveBeenCalledOnce();
    expect(onConfirm).not.toHaveBeenCalled();
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
  });

  it('confirms with the action button and closes', async () => {
    const user = userEvent.setup();
    const { onConfirm } = renderAlertDialog();

    await open(user);
    const action = screen.getByRole('button', { name: 'Supprimer' });
    expect(action).toHaveClass('bg-danger-solid');
    await user.click(action);
    expect(onConfirm).toHaveBeenCalledOnce();
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
  });

  it('closes on Escape without confirming', async () => {
    const user = userEvent.setup();
    const { onConfirm } = renderAlertDialog();

    await open(user);
    await user.keyboard('{Escape}');
    expect(screen.queryByRole('alertdialog')).not.toBeInTheDocument();
    expect(onConfirm).not.toHaveBeenCalled();
  });
});

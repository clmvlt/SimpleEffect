import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';

import { Toaster, toast } from './toast';

afterEach(() => {
  act(() => {
    toast.dismiss();
  });
});

describe('Toaster', () => {
  it('is a named notification region', () => {
    render(<Toaster label="Notifications" />);
    expect(screen.getByRole('region', { name: 'Notifications' })).toBeInTheDocument();
  });

  it('shows the toasts sent with toast()', async () => {
    render(<Toaster label="Notifications" />);

    act(() => {
      toast.success('Export terminé', { description: 'interview.mp4 · 1 min 12 s' });
    });

    expect(await screen.findByText('Export terminé')).toBeInTheDocument();
    expect(screen.getByText('interview.mp4 · 1 min 12 s')).toBeInTheDocument();
    expect(screen.getByText('Export terminé').closest('[data-sonner-toast]')).toHaveAttribute(
      'data-type',
      'success',
    );
  });

  it('runs a toast action', async () => {
    const user = userEvent.setup();
    const onUndo = vi.fn();
    render(<Toaster label="Notifications" />);

    act(() => {
      toast('Clip supprimé', { action: { label: 'Annuler', onClick: onUndo } });
    });
    await user.click(await screen.findByRole('button', { name: 'Annuler' }));
    expect(onUndo).toHaveBeenCalledOnce();
  });

  it('labels the close button only when closeLabel is given', async () => {
    render(<Toaster label="Notifications" closeLabel="Fermer la notification" />);

    act(() => {
      toast.info('Proxy en cours de création');
    });
    expect(
      await screen.findByRole('button', { name: 'Fermer la notification' }),
    ).toBeInTheDocument();
  });
});

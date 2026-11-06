import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';

import { Label } from './label';

describe('Label', () => {
  it('names the control it points to with htmlFor', () => {
    render(
      <>
        <Label htmlFor="sequence-name">Nom de la séquence</Label>
        <input id="sequence-name" />
      </>,
    );
    expect(screen.getByRole('textbox', { name: 'Nom de la séquence' })).toBeInTheDocument();
  });

  it('focuses its control when clicked', async () => {
    const user = userEvent.setup();
    render(
      <>
        <Label htmlFor="sequence-name">Nom de la séquence</Label>
        <input id="sequence-name" />
      </>,
    );
    await user.click(screen.getByText('Nom de la séquence'));
    expect(screen.getByRole('textbox')).toHaveFocus();
  });
});

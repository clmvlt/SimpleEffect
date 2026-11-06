import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { createRef } from 'react';
import { describe, expect, it } from 'vitest';

import { Input } from './input';

describe('Input', () => {
  it('is a text field named by its label that accepts typing', async () => {
    const user = userEvent.setup();
    render(
      <>
        <label htmlFor="name">Nom de la séquence</label>
        <Input id="name" />
      </>,
    );
    const input = screen.getByRole('textbox', { name: 'Nom de la séquence' });
    await user.type(input, 'Interview');
    expect(input).toHaveValue('Interview');
  });

  it('exposes the invalid state to assistive technologies and styles', () => {
    render(<Input aria-label="Durée" aria-invalid />);
    const input = screen.getByRole('textbox', { name: 'Durée' });
    expect(input).toBeInvalid();
    expect(input).toHaveClass('aria-invalid:border-danger');
  });

  it('maps the size prop to a control height and forwards refs', () => {
    const ref = createRef<HTMLInputElement>();
    render(<Input aria-label="Nom" size="md" ref={ref} />);
    expect(screen.getByRole('textbox')).toHaveClass('h-control-md');
    expect(ref.current).toBeInstanceOf(HTMLInputElement);
  });
});
